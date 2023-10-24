#![allow(dead_code)]

use std::sync::OnceLock;

use axum::async_trait;
use axum::extract::rejection::TypedHeaderRejection;
use axum::extract::{FromRef, FromRequestParts, TypedHeader};
use axum::headers::authorization::Bearer;
use axum::headers::Authorization;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, RequestPartsExt};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::database::Database;

// Allow 15 minute token windows for now, this is likely to change in the future
pub const EXPIRATION_WINDOW_SECS: u64 = 900;

static KEY_ID_VALIDATOR: OnceLock<regex::Regex> = OnceLock::new();

const KEY_ID_REGEX: &str = r"^[0-9a-f]{2}(:[0-9a-f]{2}){19}$";

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ApiToken {
    #[serde(rename = "iat")]
    pub issued_at: u64,

    #[serde(rename = "nonce")]
    pub nonce: Option<String>,

    #[serde(rename = "exp")]
    pub expiration: u64,

    #[serde(rename = "nbf")]
    pub not_before: u64,

    #[serde(rename = "aud")]
    pub audience: String,

    #[serde(rename = "sub")]
    pub subject: String,
}

impl ApiToken {
    pub fn subject(&self) -> String {
        self.subject.clone()
    }
}

pub struct ApiIdentity {
    pub account_id: String,
    pub user_id: String,
    pub device_api_key_id: String,
    pub device_api_key_fingerprint: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for ApiIdentity
where
    Database: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ApiIdentityError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let key_regex = KEY_ID_VALIDATOR.get_or_init(|| regex::Regex::new(KEY_ID_REGEX).unwrap());

        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(ApiIdentityError::MissingHeader)?;
        let mut token_validator = Validation::new(Algorithm::ES384);

        // Allow +/- 20 sec clock skew off the expiration and not before time
        token_validator.leeway = 20;

        // Restrict audience as our clients will use the same API key for authorization to multiple
        // services
        //token_validator.set_audience(&["banyan-platform"]);

        // Require all of our keys except for the attestations and proofs
        token_validator.set_required_spec_claims(&["aud", "exp", "nbf", "sub", "iat"]);

        let token = bearer.token();
        let header_data = decode_header(token).map_err(ApiIdentityError::FormatError)?;

        let key_id = match header_data.kid {
            Some(key_id) if key_regex.is_match(key_id.as_str()) => key_id,
            Some(_) => return Err(ApiIdentityError::BadKeyFormat),
            None => return Err(ApiIdentityError::UnidentifiedKey),
        };

        let database = Database::from_ref(state);

        let db_device_api_key = sqlx::query_as!(
            DeviceApiKey,
            r#"SELECT dak.id, a.id as account_id, a.userId as user_id, dak.pem
                   FROM device_api_keys AS dak
                   JOIN accounts AS a ON dak.account_id = a.id
                   WHERE dak.fingerprint = $1;"#,
            key_id
        )
        .fetch_one(&database)
        .await
        .map_err(ApiIdentityError::DeviceApiKeyNotFound)?;

        let key = DecodingKey::from_ec_pem(db_device_api_key.pem.as_bytes())
            .map_err(|err| ApiIdentityError::DatabaseCorrupt(db_device_api_key.id.clone(), err))?;

        // TODO: we probably want to use device keys to sign this instead of a
        // static AES key, this works for now
        let token_data = decode::<ApiToken>(token, &key, &token_validator)
            .map_err(ApiIdentityError::FormatError)?;

        let claims = token_data.claims;

        match claims.expiration.checked_sub(claims.not_before) {
            Some(duration) => {
                if duration > EXPIRATION_WINDOW_SECS {
                    return Err(ApiIdentityError::ExtremeTokenValidity);
                }
            }
            None => {
                // the not before value was after the expiration, a negative duration is never
                // valid and we should immediate reject it
                return Err(ApiIdentityError::NeverValid);
            }
        }

        if db_device_api_key.account_id != claims.subject {
            return Err(ApiIdentityError::MismatchedSubject);
        }

        let api_identity = ApiIdentity {
            account_id: claims.subject,
            user_id: db_device_api_key.user_id,
            device_api_key_id: db_device_api_key.id,
            device_api_key_fingerprint: key_id,
        };

        Ok(api_identity)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ApiIdentityError {
    #[error("key format in JWT header wasn't valid")]
    BadKeyFormat,

    #[error("public key '{0}' stored in database is corrupted")]
    DatabaseCorrupt(String, jsonwebtoken::errors::Error),

    #[error("unable to lookup device API key in database")]
    DeviceApiKeyNotFound(sqlx::Error),

    #[error("the provided token's validity range is outside our allowed range")]
    ExtremeTokenValidity,

    #[error("format of the provide bearer token didn't meet our requirements")]
    FormatError(jsonwebtoken::errors::Error),

    #[error("no Authorization header was present in request to protected route")]
    MissingHeader(TypedHeaderRejection),

    #[error("token had a valid signature but the key was not owned by the represented subject")]
    MismatchedSubject,

    #[error("authorization token doesn't become valid until after it has already expired")]
    NeverValid,

    #[error(
        "header didn't include kid required to lookup the appropriate authentication mechanism"
    )]
    UnidentifiedKey,
}

impl IntoResponse for ApiIdentityError {
    fn into_response(self) -> axum::response::Response {
        // Report to the CLI, not to the end user. Don't give attackers knowledge of what we didn't
        // like about their request
        use crate::utils::collect_error_messages;
        tracing::error!("authentication failed: {:?}", &collect_error_messages(self));

        let err_msg = serde_json::json!({ "msg": "not authorized" });
        (StatusCode::UNAUTHORIZED, Json(err_msg)).into_response()
    }
}

#[derive(sqlx::FromRow)]
struct DeviceApiKey {
    id: String,
    account_id: String,
    user_id: String,
    pem: String,
}
