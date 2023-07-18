use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

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

// Allow 15 minute token windows for now, this is likely to change in the future
pub const EXPIRATION_WINDOW_SECS: u64 = 900;

// todo: extract this from state, populate this from the env
pub const TESTING_API_KEY: &str = "This key will come from the environment";

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ApiToken {
    #[serde(rename = "nnc")]
    pub nonce: Option<String>,

    #[serde(rename = "exp")]
    pub expiration: u64,

    #[serde(rename = "nbf")]
    pub not_before: u64,

    #[serde(rename = "aud")]
    pub audience: String,

    // todo: may be able to get more structured about the subject to go along with attenuations.
    #[serde(rename = "sub")]
    pub subject: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for ApiToken
where
    S: Send + Sync,
{
    type Rejection = ApiKeyAuthorizationError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(ApiKeyAuthorizationError::missing_header)?;

        let key = DecodingKey::from_secret(TESTING_API_KEY.as_ref());
        let mut token_validator = Validation::new(Algorithm::HS256);

        // Allow +/- 20 sec clock skew off the expiration and not before time
        token_validator.leeway = 20;

        // Restrict audience as our clients will use the same API key for authorization to multiple
        // services
        token_validator.set_audience(&["banyan-core"]);

        // Require all of our keys except for the attestations and proofs
        token_validator.set_required_spec_claims(&["aud", "exp", "nbf", "sub"]);

        let token = bearer.token();

        let header_data = decode_header(&token)
            .map_err(ApiKeyAuthorizationError::decode_failed)?;

        let looked_up_key = match header_data.kid {
            Some(key_id) => key_id.to_string(),
            None => return Err(ApiKeyAuthorizationError::unidentified_key()),
        };

        // todo: we probably want to use device keys to sign this instead of a
        // static AES key, this works for now
        let token_data = decode::<ApiToken>(token, &key, &token_validator)
            .map_err(ApiKeyAuthorizationError::decode_failed)?;

        let claims = token_data.claims;

        match claims.expiration.checked_sub(claims.not_before) {
            Some(duration) => {
                if duration > EXPIRATION_WINDOW_SECS {
                    return Err(ApiKeyAuthorizationError {
                        kind: ApiKeyAuthorizationErrorKind::ExtremeTokenValidity,
                    });
                }
            }
            None => {
                // the not before value was after the expiration, a negative duration is never
                // valid and we should immediate reject it
                return Err(ApiKeyAuthorizationError {
                    kind: ApiKeyAuthorizationErrorKind::NeverValid,
                });
            }
        }

        Ok(claims)
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct ApiKeyAuthorizationError {
    kind: ApiKeyAuthorizationErrorKind,
}

impl ApiKeyAuthorizationError {
    fn decode_failed(err: jsonwebtoken::errors::Error) -> Self {
        use jsonwebtoken::errors::ErrorKind::*;

        let kind = match err.kind() {
            Base64(_)
            | InvalidAudience
            | InvalidIssuer
            | InvalidSubject
            | InvalidToken
            | MissingAlgorithm
            | MissingRequiredClaim(_) => ApiKeyAuthorizationErrorKind::FormatError(err),
            ExpiredSignature | ImmatureSignature | InvalidAlgorithm | InvalidAlgorithmName
            | InvalidSignature => ApiKeyAuthorizationErrorKind::MaliciousConstruction(err),
            InvalidEcdsaKey | InvalidKeyFormat | InvalidRsaKey(_) => {
                ApiKeyAuthorizationErrorKind::InternalCryptographyIssue(err)
            }
            _ => ApiKeyAuthorizationErrorKind::UnknownTokenError(err),
        };

        Self { kind }
    }

    fn missing_header(err: TypedHeaderRejection) -> Self {
        Self {
            kind: ApiKeyAuthorizationErrorKind::MissingHeader(err),
        }
    }

    fn unidentified_key() -> Self {
        Self {
            kind: ApiKeyAuthorizationErrorKind::UnidentifiedKey,
        }
    }
}

impl Display for ApiKeyAuthorizationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use ApiKeyAuthorizationErrorKind::*;

        let msg = match self.kind {
            ExtremeTokenValidity => "the provided token's validity range is outside our allowed range",
            FormatError(_) => "format of the provide bearer token didn't meet our requirements",
            InternalCryptographyIssue(_) => "there was an internal cryptographic issue due too a code or configuration issue",
            MaliciousConstruction(_) => "the provided token was invalid in a way that indicates an attack is likely occurring",
            MissingHeader(_) => "no Authorization header was present in request to protected route",
            NeverValid => "authorization token doesn't become valid until after it has already expired",
            UnidentifiedKey => "header didn't include kid required to lookup the appropriate authentication mechanism",
            UnknownTokenError(_) => "an unexpected error edge case occurred around an authentation token",
        };

        f.write_str(msg)
    }
}

impl std::error::Error for ApiKeyAuthorizationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use ApiKeyAuthorizationErrorKind::*;

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

impl IntoResponse for ApiKeyAuthorizationError {
    fn into_response(self) -> axum::response::Response {
        // Report to the CLI, not to the end user. Don't give attackers knowledge of what we didn't
        // like about their request
        use crate::util::collect_error_messages;
        tracing::error!("authentication failed: {:?}", &collect_error_messages(self));

        let err_msg = serde_json::json!({ "status": "not authorized" });
        (StatusCode::UNAUTHORIZED, Json(err_msg)).into_response()
    }
}

#[derive(Debug)]
enum ApiKeyAuthorizationErrorKind {
    ExtremeTokenValidity,
    FormatError(jsonwebtoken::errors::Error),
    InternalCryptographyIssue(jsonwebtoken::errors::Error),
    MaliciousConstruction(jsonwebtoken::errors::Error),
    MissingHeader(TypedHeaderRejection),
    NeverValid,
    UnidentifiedKey,
    UnknownTokenError(jsonwebtoken::errors::Error),
}
