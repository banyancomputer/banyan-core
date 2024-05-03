use axum::extract::rejection::TypedHeaderRejection;
use axum::extract::{FromRef, FromRequestParts, TypedHeader};
use axum::headers::authorization::Bearer;
use axum::headers::Authorization;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{async_trait, Json, RequestPartsExt};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use super::{EXPIRATION_WINDOW, KEY_ID_REGEX, KEY_ID_VALIDATOR};
use crate::database::Database;

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

#[derive(Default)]
pub struct StorageProviderIdentity {
    pub id: String,
    pub name: String,
    pub staging: bool,
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
        token_validator.set_required_spec_claims(&["exp", "nbf", "sub", "iat"]);

        let token = bearer.token();
        let header_data =
            decode_header(token).map_err(StorageProviderIdentityError::FormatError)?;

        let key_id = match header_data.kid {
            Some(key_id) if key_regex.is_match(key_id.as_str()) => key_id,
            Some(unknown_kid) => {
                tracing::warn!("encountered key not matching expected format: {unknown_kid}");
                return Err(StorageProviderIdentityError::BadKeyFormat);
            }
            None => return Err(StorageProviderIdentityError::UnidentifiedKey),
        };

        let mut conn = Database::from_ref(state)
            .acquire()
            .await
            .map_err(StorageProviderIdentityError::DatabaseConnection)?;
        let maybe_storage_host = sqlx::query_as!(
            StorageHost,
            "SELECT id, name, pem, staging FROM storage_hosts WHERE fingerprint = $1",
            key_id
        )
        .fetch_optional(&mut *conn)
        .await
        .map_err(StorageProviderIdentityError::DatabaseUnavailable)?;

        let storage_host = match maybe_storage_host {
            Some(sh) => sh,
            None => {
                return Err(StorageProviderIdentityError::StorageHostNotFound);
            }
        };

        let key = match DecodingKey::from_ec_pem(storage_host.pem.as_bytes()) {
            Ok(k) => k,
            Err(err) => {
                tracing::error!("storage host for public key was invalid: {err}");
                return Err(StorageProviderIdentityError::FormatError(err));
            }
        };

        let token_data = match decode::<StorageHostToken>(token, &key, &token_validator) {
            Ok(td) => td,
            Err(err) => {
                tracing::error!("failed to validate the JWT with our given parameters: {err}");
                return Err(StorageProviderIdentityError::FormatError(err));
            }
        };

        let claims = token_data.claims;

        match claims
            .expiration
            .checked_sub(claims.not_before)
            .map(std::time::Duration::from_secs)
        {
            Some(duration) => {
                if duration > EXPIRATION_WINDOW {
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
            name: storage_host.name,
            staging: storage_host.staging,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StorageProviderIdentityError {
    #[error("key format in JWT header wasn't valid")]
    BadKeyFormat,

    #[error("unable to lookup identity in database: {0}")]
    DatabaseUnavailable(sqlx::Error),

    #[error("could not acquire connection: {0}")]
    DatabaseConnection(sqlx::Error),

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
    staging: bool,
}
