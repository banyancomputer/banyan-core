use axum::async_trait;
use axum::extract::{FromRef, FromRequestParts};
use jsonwebtoken::EncodingKey;

pub struct ServiceKey(pub(crate) EncodingKey);

#[async_trait]
impl<S> FromRequestParts<S> for ServiceKey
where
    EncodingKey: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        _parts: &mut http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(EncodingKey::from_ref(state)))
    }
}
