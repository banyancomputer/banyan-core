use std::sync::OnceLock;

use axum::async_trait;
use axum::extract::rejection::TypedHeaderRejection;
use axum::extract::{FromRef, FromRequestParts, TypedHeader};
use axum::headers::authorization::Bearer;
use axum::headers::Authorization;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Json, RequestPartsExt};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::database::Database;

// Allow 15 minute token windows for now, this is likely to change in the future
pub const EXPIRATION_WINDOW_SECS: u64 = 900;

static KEY_ID_VALIDATOR: OnceLock<regex::Regex> = OnceLock::new();

const KEY_ID_REGEX: &str = r"^[0-9a-f]{2}(:[0-9a-f]{2}){31}$";

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StorageHostToken {
    #[serde(rename = "iat")]
    pub issued_at: u64,

    #[serde(rename = "nonce")]
    pub nonce: Option<String>,

    #[serde(rename = "exp")]
    pub expiration: u64,

    #[serde(rename = "nbf")]
    pub not_before: u64,

    #[serde(rename = "aud")]
    pub audience: Vec<String>,

    #[serde(rename = "sub")]
    pub subject: String,
}

pub struct StorageProviderIdentity {
    pub id: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for StorageProviderIdentity
where
    Database: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = StorageProviderIdentityError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let key_regex = KEY_ID_VALIDATOR.get_or_init(|| regex::Regex::new(KEY_ID_REGEX).unwrap());

        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(StorageProviderIdentityError::MissingHeader)?;
        let mut token_validator = Validation::new(Algorithm::ES384);

        // Allow +/- 20 sec clock skew off the expiration and not before time
        token_validator.leeway = 20;

        // Restrict audience as our clients will use the same API key for authorization to multiple
        // services
        token_validator.set_audience(&["banyan-platform"]);

        // Require all of our keys except for the attestations and proofs
        token_validator.set_required_spec_claims(&["aud", "exp", "nbf", "sub", "iat"]);

        let token = bearer.token();
        let header_data =
            decode_header(token).map_err(StorageProviderIdentityError::FormatError)?;

        let key_id = match header_data.kid {
            Some(key_id) if key_regex.is_match(key_id.as_str()) => key_id,
            Some(_) => return Err(StorageProviderIdentityError::BadKeyFormat),
            None => return Err(StorageProviderIdentityError::UnidentifiedKey),
        };

        let database = Database::from_ref(state);

        let maybe_storage_host = sqlx::query_as!(
            StorageHost,
            "SELECT id, name, pem FROM storage_hosts WHERE fingerprint = $1",
            key_id
        )
        .fetch_optional(&database)
        .await
        .map_err(StorageProviderIdentityError::DatabaseUnavailable)?;

        let storage_host = match maybe_storage_host {
            Some(sh) => sh,
            None => {
                return Err(StorageProviderIdentityError::StorageHostNotFound);
            }
        };

        let key = DecodingKey::from_ec_pem(storage_host.pem.as_bytes())
            .map_err(StorageProviderIdentityError::FormatError)?;
        let token_data = decode::<StorageHostToken>(token, &key, &token_validator)
            .map_err(StorageProviderIdentityError::FormatError)?;

        let claims = token_data.claims;

        match claims.expiration.checked_sub(claims.not_before) {
            Some(duration) => {
                if duration > EXPIRATION_WINDOW_SECS {
                    return Err(StorageProviderIdentityError::ExtremeTokenValidity);
                }
            }
            None => {
                // the not before value was after the expiration, a negative duration is never
                // valid and we should immediate reject it
                return Err(StorageProviderIdentityError::NeverValid);
            }
        }

        if storage_host.name != claims.subject {
            return Err(StorageProviderIdentityError::MismatchedSubject);
        }

        Ok(StorageProviderIdentity {
            id: storage_host.id,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StorageProviderIdentityError {
    #[error("key format in JWT header wasn't valid")]
    BadKeyFormat,

    #[error("unable to lookup identity in database: {0}")]
    DatabaseUnavailable(sqlx::Error),

    #[error("unable to lookup storage host in database")]
    StorageHostNotFound,

    #[error("the provided token's validity range is outside our allowed range")]
    ExtremeTokenValidity,

    #[error("format of the provide bearer token didn't meet our requirements: {0}")]
    FormatError(jsonwebtoken::errors::Error),

    #[error("token had a valid signature but the key was not owned by the represented subject")]
    MismatchedSubject,

    #[error("no authorization header was present in request to protected route: {0}")]
    MissingHeader(TypedHeaderRejection),

    #[error("authorization token doesn't become valid until after it has already expired")]
    NeverValid,

    #[error(
        "header didn't include kid required to lookup the appropriate authentication mechanism"
    )]
    UnidentifiedKey,
}

impl IntoResponse for StorageProviderIdentityError {
    fn into_response(self) -> Response {
        use crate::utils::collect_error_messages;
        tracing::error!("authentication failed: {:?}", &collect_error_messages(self));

        let err_msg = serde_json::json!({ "status": "not authorized" });
        (StatusCode::UNAUTHORIZED, Json(err_msg)).into_response()
    }
}

#[derive(sqlx::FromRow)]
struct StorageHost {
    id: String,
    name: String,
    pem: String,
}
