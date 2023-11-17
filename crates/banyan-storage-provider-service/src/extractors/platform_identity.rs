



use axum::extract::{FromRef, FromRequestParts, TypedHeader};
use axum::headers::authorization::Bearer;
use axum::headers::Authorization;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{async_trait, Json, RequestPartsExt};
use http::request::Parts;
use jwt_simple::prelude::*;




use crate::app::PlatformVerificationKey;
use crate::extractors::fingerprint_validator;

/// Defines the maximum length of time we consider any individual token valid in seconds. If the
/// expiration is still in the future, but it was issued more than this many seconds in the past
/// we'll reject the token even if its otherwise valid.
const MAXIMUM_TOKEN_AGE: u64 = 900;

pub struct PlatformIdentity;

#[async_trait]
impl<S> FromRequestParts<S> for PlatformIdentity
where
    PlatformVerificationKey: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = PlatformIdentityError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| Self::Rejection::MissingHeader)?;

        let raw_token = bearer.token();

        let unvalidated_header =
            Token::decode_metadata(raw_token).map_err(Self::Rejection::CorruptHeader)?;
        match unvalidated_header.key_id() {
            Some(kid) if fingerprint_validator().is_match(kid) => kid.to_string(),
            Some(_) => return Err(Self::Rejection::InvalidKeyId),
            None => return Err(Self::Rejection::MissingKeyId),
        };

        let platform_verification_key = PlatformVerificationKey::from_ref(state);

        let verification_options = VerificationOptions {
            accept_future: false,
            // TODO: this might not be a quite right, but it's probably fine for now
            //allowed_audiences: Some(HashSet::from_strings(&["banyan-platform"])),
            max_validity: Some(Duration::from_secs(MAXIMUM_TOKEN_AGE)),
            time_tolerance: Some(Duration::from_secs(15)),
            ..Default::default()
        };

        let claims = platform_verification_key
            .verify_token::<NoCustomClaims>(raw_token, Some(verification_options))
            .map_err(Self::Rejection::ValidationFailed)?;

        // annoyingly jwt-simple doesn't use the correct encoding for this... we can support both
        // though and maybe we can fix upstream so it follows the spec
        let nonce = claims.nonce.ok_or(Self::Rejection::BadNonce)?;
        if nonce.len() < 12 {
            return Err(Self::Rejection::BadNonce);
        }

        Ok(Self)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PlatformIdentityError {
    #[error("nonce wasn't present or insufficiently long")]
    BadNonce,

    #[error("unable to decode bearer token metadata")]
    CorruptHeader(jwt_simple::Error),

    #[error("bearer token key ID does not conform to our expectations")]
    InvalidKeyId,

    #[error("authentication header wasn't present")]
    MissingHeader,

    #[error("no token key ID was provided")]
    MissingKeyId,

    #[error("failed to validate JWT with provided key and parameters")]
    ValidationFailed(jwt_simple::Error),
}

impl IntoResponse for PlatformIdentityError {
    fn into_response(self) -> Response {
        use PlatformIdentityError::*;

        match &self {
            BadNonce | CorruptHeader(_) | InvalidKeyId | MissingKeyId | ValidationFailed(_) => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({ "msg": "invalid request" });
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            MissingHeader => {
                let err_msg = serde_json::json!({ "msg": "authentication required" });
                (StatusCode::UNAUTHORIZED, Json(err_msg)).into_response()
            }
        }
    }
}
