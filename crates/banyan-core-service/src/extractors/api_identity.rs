use axum::extract::rejection::TypedHeaderRejection;
use axum::extract::{FromRef, FromRequestParts, TypedHeader};
use axum::headers::authorization::Bearer;
use axum::headers::Authorization;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{async_trait, Json, RequestPartsExt};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{EXPIRATION_WINDOW, KEY_ID_REGEX, KEY_ID_VALIDATOR};
use crate::database::models::UserKey;
use crate::database::Database;

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

/// Extracted identity from an API request made with a client-signed JWT
pub struct ApiIdentity {
    /// The user id of the user who owns the API key
    user_id: Uuid,
    /// The hex formatted fingerprint of the API key used to sign the JWT
    key_fingerprint: String,
}

impl ApiIdentity {
    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    pub fn key_fingerprint(&self) -> &str {
        &self.key_fingerprint
    }

    pub fn ticket_subject(&self) -> String {
        format!("{}@{}", self.user_id(), self.key_fingerprint())
    }
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

        // TODO: eventually implement aud restriction
        // Restrict audience as our clients will use the same API key for authorization to multiple
        // services
        token_validator.set_audience(&["banyan-platform"]);
        // Require all of our keys except for the attestations and proofs
        token_validator.set_required_spec_claims(&["exp", "nbf", "sub", "iat"]);

        let token = bearer.token();
        let header_data = decode_header(token).map_err(ApiIdentityError::FormatError)?;

        let key_id = match header_data.kid {
            Some(key_id) if key_regex.is_match(key_id.as_str()) => key_id,
            Some(_) => return Err(ApiIdentityError::BadKeyFormat),
            None => return Err(ApiIdentityError::UnidentifiedKey),
        };

        let database = Database::from_ref(state);

        let db_user_key = UserKey::from_fingerprint(&database, &key_id)
            .await
            .map_err(ApiIdentityError::DeviceApiKeyNotFound)?;

        let key = DecodingKey::from_ec_pem(db_user_key.pem.as_bytes())
            .map_err(|err| ApiIdentityError::DatabaseCorrupt(db_user_key.id.clone(), err))?;

        // TODO: we probably want to use device keys to sign this instead of a
        // static AES key, this works for now
        let token_data = decode::<ApiToken>(token, &key, &token_validator)
            .map_err(ApiIdentityError::FormatError)?;

        let claims = token_data.claims;

        match claims
            .expiration
            .checked_sub(claims.not_before)
            .map(std::time::Duration::from_secs)
        {
            Some(duration) => {
                if duration > EXPIRATION_WINDOW {
                    return Err(ApiIdentityError::ExtremeTokenValidity);
                }
            }
            None => {
                // the not before value was after the expiration, a negative duration is never
                // valid and we should immediate reject it
                return Err(ApiIdentityError::NeverValid);
            }
        }

        if db_user_key.user_id != claims.subject {
            return Err(ApiIdentityError::MismatchedSubject);
        }

        let user_id =
            Uuid::parse_str(&claims.subject).map_err(ApiIdentityError::DatabaseUuidCorrupt)?;
        let key_fingerprint = key_id.clone();
        let api_identity = ApiIdentity {
            user_id,
            key_fingerprint,
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

    #[error("uuid '{0}' stored in database is corrupted")]
    DatabaseUuidCorrupt(uuid::Error),

    #[error("unable to lookup device API key in database")]
    DeviceApiKeyNotFound(sqlx::Error),

    #[error("the provided token's validity range is outside our allowed range")]
    ExtremeTokenValidity,

    #[error("format of the provided bearer token didn't meet our requirements")]
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
    user_id: String,
    pem: String,
}

#[cfg(test)]
pub mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::database::test_helpers;

    pub struct ApiIdentityBuilder {
        pub user_id: Uuid,
        pub key_fingerprint: String,
    }

    impl Default for ApiIdentityBuilder {
        fn default() -> Self {
            Self {
                user_id: Uuid::new_v4(),
                key_fingerprint: String::default(),
            }
        }
    }
    impl ApiIdentityBuilder {
        pub fn build(self) -> ApiIdentity {
            ApiIdentity {
                user_id: self.user_id,
                key_fingerprint: self.key_fingerprint,
            }
        }
    }

    #[tokio::test]
    async fn test_api_identity_builder() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.acquire().await.expect("connection");

        let user_id = test_helpers::sample_user(&mut conn, "test@example.com").await;
        let api_identity = test_helpers::get_or_create_identity(&mut conn, &user_id).await;

        assert_eq!(api_identity.user_id().to_string(), user_id);

        let retry_identity = test_helpers::get_or_create_identity(&mut conn, &user_id).await;
        assert_eq!(retry_identity.key_fingerprint, api_identity.key_fingerprint);
    }
}
