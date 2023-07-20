use axum::async_trait;
use axum::extract::{FromRef, FromRequestParts, TypedHeader};
use axum::headers::authorization::Bearer;
use axum::headers::Authorization;
use axum::http::request::Parts;
use axum::RequestPartsExt;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::extractors::api_token::ApiKeyAuthorizationError;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FakeToken {
    #[serde(rename = "exp")]
    pub expiration: u64,

    #[serde(rename = "sub")]
    pub subject: String,
}

#[async_trait]
impl<S> FromRequestParts<S> for FakeToken
where
    DecodingKey: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ApiKeyAuthorizationError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(ApiKeyAuthorizationError::missing_header)?;

        let mut token_validator = Validation::new(Algorithm::ES384);
        token_validator.set_required_spec_claims(&["exp", "sub"]);

        let key = DecodingKey::from_ref(state);
        let token_data = decode::<FakeToken>(bearer.token(), &key, &token_validator)
            .map_err(ApiKeyAuthorizationError::decode_failed)?;

        Ok(token_data.claims)
    }
}
