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
use jsonwebtoken::decode_header;
use serde::{Deserialize, Serialize};

use crate::extractors::DbConn;

static KEY_ID_VALIDATOR: OnceLock<regex::Regex> = OnceLock::new();
const KEY_ID_REGEX: &str = r"^[0-9a-f]{2}(:[0-9a-f]{2}){19}$";

#[derive(Deserialize, Serialize)]
pub struct ApiTokenKid(String);

impl ApiTokenKid {
    pub fn kid(&self) -> &str {
        self.0.as_str()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for ApiTokenKid
where
    DbConn: FromRequestParts<S>,
    S: Send + Sync,
{
    type Rejection = ApiKeyIdAuthorizationError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let key_regex = KEY_ID_VALIDATOR.get_or_init(|| regex::Regex::new(KEY_ID_REGEX).unwrap());

        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(ApiKeyIdAuthorizationError::missing_header)?;

        let token = bearer.token();
        let header_data =
            decode_header(token).map_err(ApiKeyIdAuthorizationError::decode_failed)?;

        let kid = match header_data.kid {
            Some(key_id) if key_regex.is_match(key_id.as_str()) => key_id,
            Some(_) => return Err(ApiKeyIdAuthorizationError::bad_key_format()),
            None => return Err(ApiKeyIdAuthorizationError::unidentified_key()),
        };

        Ok(Self(kid))
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct ApiKeyIdAuthorizationError {
    kind: ApiKeyIdAuthorizationErrorKind,
}

impl ApiKeyIdAuthorizationError {
    pub fn bad_key_format() -> Self {
        Self {
            kind: ApiKeyIdAuthorizationErrorKind::BadKeyFormat,
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
            | MissingRequiredClaim(_) => ApiKeyIdAuthorizationErrorKind::FormatError(err),
            ExpiredSignature | ImmatureSignature | InvalidAlgorithm | InvalidAlgorithmName
            | InvalidSignature => ApiKeyIdAuthorizationErrorKind::MaliciousConstruction(err),
            InvalidEcdsaKey | InvalidKeyFormat | InvalidRsaKey(_) => {
                ApiKeyIdAuthorizationErrorKind::InternalCryptographyIssue(err)
            }
            _ => ApiKeyIdAuthorizationErrorKind::UnknownTokenError(err),
        };

        Self { kind }
    }

    pub fn missing_header(err: TypedHeaderRejection) -> Self {
        Self {
            kind: ApiKeyIdAuthorizationErrorKind::MissingHeader(err),
        }
    }

    pub fn unidentified_key() -> Self {
        Self {
            kind: ApiKeyIdAuthorizationErrorKind::UnidentifiedKey,
        }
    }
}

impl Display for ApiKeyIdAuthorizationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use ApiKeyIdAuthorizationErrorKind::*;

        let msg = match self.kind {
            BadKeyFormat => "key format in JWT header wasn't valid",
            FormatError(_) => "format of the provide bearer token didn't meet our requirements",
            InternalCryptographyIssue(_) => "there was an internal cryptographic issue due too a code or configuration issue",
            MaliciousConstruction(_) => "the provided token was invalid in a way that indicates an attack is likely occurring",
            MissingHeader(_) => "no Authorization header was present in request to protected route",
            UnidentifiedKey => "header didn't include kid required to lookup the appropriate authentication mechanism",
            UnknownTokenError(_) => "an unexpected error edge case occurred around an authentation token",
        };

        f.write_str(msg)
    }
}

// dunno if this is needed...
impl std::error::Error for ApiKeyIdAuthorizationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use ApiKeyIdAuthorizationErrorKind::*;

        match &self.kind {
            FormatError(err) => Some(err),
            InternalCryptographyIssue(err) => Some(err),
            MaliciousConstruction(err) => Some(err),
            MissingHeader(err) => Some(err),
            UnknownTokenError(err) => Some(err),
            _ => None,
        }
    }
}

impl IntoResponse for ApiKeyIdAuthorizationError {
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
enum ApiKeyIdAuthorizationErrorKind {
    BadKeyFormat,
    FormatError(jsonwebtoken::errors::Error),
    InternalCryptographyIssue(jsonwebtoken::errors::Error),
    MaliciousConstruction(jsonwebtoken::errors::Error),
    MissingHeader(TypedHeaderRejection),
    UnidentifiedKey,
    UnknownTokenError(jsonwebtoken::errors::Error),
}
