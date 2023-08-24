use std::fmt::{self, Display, Formatter};
use std::sync::OnceLock;

use axum::async_trait;
use axum::extract::rejection::TypedHeaderRejection;
use axum::extract::{FromRequestParts, TypedHeader};
use axum::headers::authorization::Bearer;
use axum::headers::Authorization;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, RequestPartsExt};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::extractors::DbConn;

// Allow 15 minute token windows for now, this is likely to change in the future
pub const EXPIRATION_WINDOW_SECS: u64 = 900;

static KEY_ID_VALIDATOR: OnceLock<regex::Regex> = OnceLock::new();
const KEY_ID_REGEX: &str = r"^[0-9a-f]{2}(:[0-9a-f]{2}){19}$";

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
    pub audience: String,

    #[serde(rename = "sub")]
    pub subject: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for StorageHostToken
where
    DbConn: FromRequestParts<S>,
    S: Send + Sync,
{
    type Rejection = StorageHostKeyAuthorizationError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let key_regex = KEY_ID_VALIDATOR.get_or_init(|| regex::Regex::new(KEY_ID_REGEX).unwrap());

        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(StorageHostKeyAuthorizationError::missing_header)?;
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
            decode_header(token).map_err(StorageHostKeyAuthorizationError::decode_failed)?;

        let key_id = match header_data.kid {
            Some(key_id) if key_regex.is_match(key_id.as_str()) => key_id,
            Some(_) => return Err(StorageHostKeyAuthorizationError::bad_key_format()),
            None => return Err(StorageHostKeyAuthorizationError::unidentified_key()),
        };

        let mut db_conn = DbConn::from_request_parts(parts, state)
            .await
            .map_err(|_| StorageHostKeyAuthorizationError::database_unavailable())?;

        let storage_host = sqlx::query_as!(
            StorageHost,
            "SELECT name, pem FROM storage_hosts WHERE fingerprint = $1",
            key_id
        )
        .fetch_one(&mut *db_conn.0)
        .await
        .map_err(StorageHostKeyAuthorizationError::storage_host_not_found)?;

        let key = DecodingKey::from_ec_pem(storage_host.pem.as_bytes()).expect("success");

        // TODO: we probably want to use device keys to sign this instead of a
        // static AES key, this works for now
        let token_data = decode::<StorageHostToken>(token, &key, &token_validator)
            .map_err(StorageHostKeyAuthorizationError::decode_failed)?;

        let claims = token_data.claims;

        match claims.expiration.checked_sub(claims.not_before) {
            Some(duration) => {
                if duration > EXPIRATION_WINDOW_SECS {
                    return Err(StorageHostKeyAuthorizationError {
                        kind: StorageHostKeyAuthorizationErrorKind::ExtremeTokenValidity,
                    });
                }
            }
            None => {
                // the not before value was after the expiration, a negative duration is never
                // valid and we should immediate reject it
                return Err(StorageHostKeyAuthorizationError {
                    kind: StorageHostKeyAuthorizationErrorKind::NeverValid,
                });
            }
        }

        if storage_host.name != claims.subject {
            return Err(StorageHostKeyAuthorizationError {
                kind: StorageHostKeyAuthorizationErrorKind::MismatchedSubject,
            });
        }

        Ok(claims)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct StorageHostKeyAuthorizationError {
    kind: StorageHostKeyAuthorizationErrorKind,
}

impl StorageHostKeyAuthorizationError {
    pub fn bad_key_format() -> Self {
        Self {
            kind: StorageHostKeyAuthorizationErrorKind::BadKeyFormat,
        }
    }

    pub fn database_unavailable() -> Self {
        Self {
            kind: StorageHostKeyAuthorizationErrorKind::DatabaseUnavailable,
        }
    }

    pub fn decode_failed(err: jsonwebtoken::errors::Error) -> Self {
        use jsonwebtoken::errors::ErrorKind::*;

        let kind = match err.kind() {
            Base64(_)
            | InvalidAudience
            | InvalidIssuer
            | InvalidSubject
            | InvalidToken
            | MissingAlgorithm
            | MissingRequiredClaim(_) => StorageHostKeyAuthorizationErrorKind::FormatError(err),
            ExpiredSignature | ImmatureSignature | InvalidAlgorithm | InvalidAlgorithmName
            | InvalidSignature => StorageHostKeyAuthorizationErrorKind::MaliciousConstruction(err),
            InvalidEcdsaKey | InvalidKeyFormat | InvalidRsaKey(_) => {
                StorageHostKeyAuthorizationErrorKind::InternalCryptographyIssue(err)
            }
            _ => StorageHostKeyAuthorizationErrorKind::UnknownTokenError(err),
        };

        Self { kind }
    }

    pub fn missing_header(err: TypedHeaderRejection) -> Self {
        Self {
            kind: StorageHostKeyAuthorizationErrorKind::MissingHeader(err),
        }
    }

    pub fn unidentified_key() -> Self {
        Self {
            kind: StorageHostKeyAuthorizationErrorKind::UnidentifiedKey,
        }
    }

    pub fn storage_host_not_found(err: sqlx::Error) -> Self {
        Self {
            kind: StorageHostKeyAuthorizationErrorKind::StorageHostNotFound(err),
        }
    }
}

impl Display for StorageHostKeyAuthorizationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use StorageHostKeyAuthorizationErrorKind::*;

        let msg = match self.kind {
            BadKeyFormat => "key format in JWT header wasn't valid",
            DatabaseUnavailable => "unable to lookup identity in database",
            StorageHostNotFound(_) => "unable to lookup storage host in database",
            ExtremeTokenValidity => "the provided token's validity range is outside our allowed range",
            FormatError(_) => "format of the provide bearer token didn't meet our requirements",
            InternalCryptographyIssue(_) => "there was an internal cryptographic issue due too a code or configuration issue",
            MaliciousConstruction(_) => "the provided token was invalid in a way that indicates an attack is likely occurring",
            MismatchedSubject => "token had a valid signature but the key was not owned by the represented subject",
            MissingHeader(_) => "no Authorization header was present in request to protected route",
            NeverValid => "authorization token doesn't become valid until after it has already expired",
            UnidentifiedKey => "header didn't include kid required to lookup the appropriate authentication mechanism",
            UnknownTokenError(_) => "an unexpected error edge case occurred around an authentation token",
        };

        f.write_str(msg)
    }
}

// dunno if this is needed...
impl std::error::Error for StorageHostKeyAuthorizationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use StorageHostKeyAuthorizationErrorKind::*;

        match &self.kind {
            FormatError(err) => Some(err),
            InternalCryptographyIssue(err) => Some(err),
            MaliciousConstruction(err) => Some(err),
            MissingHeader(err) => Some(err),
            UnknownTokenError(err) => Some(err),
            StorageHostNotFound(err) => Some(err),
            _ => None,
        }
    }
}

impl IntoResponse for StorageHostKeyAuthorizationError {
    fn into_response(self) -> axum::response::Response {
        // Report to the CLI, not to the end user. Don't give attackers knowledge of what we didn't
        // like about their request
        use crate::utils::collect_error_messages;
        tracing::error!("authentication failed: {:?}", &collect_error_messages(self));

        let err_msg = serde_json::json!({ "status": "not authorized" });
        (StatusCode::UNAUTHORIZED, Json(err_msg)).into_response()
    }
}

#[derive(Debug)]
enum StorageHostKeyAuthorizationErrorKind {
    BadKeyFormat,
    DatabaseUnavailable,
    StorageHostNotFound(sqlx::Error),
    ExtremeTokenValidity,
    FormatError(jsonwebtoken::errors::Error),
    InternalCryptographyIssue(jsonwebtoken::errors::Error),
    MaliciousConstruction(jsonwebtoken::errors::Error),
    MissingHeader(TypedHeaderRejection),
    MismatchedSubject,
    NeverValid,
    UnidentifiedKey,
    UnknownTokenError(jsonwebtoken::errors::Error),
}

#[derive(sqlx::FromRow)]
struct StorageHost {
    name: String,
    pem: String,
}
