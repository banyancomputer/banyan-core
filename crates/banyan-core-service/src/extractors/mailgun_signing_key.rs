use axum::async_trait;
use axum::extract::{FromRef, FromRequestParts};
use ring::hmac::Key as HmacKey;

pub struct MailgunSigningKey(pub(crate) HmacKey);

#[async_trait]
impl<S> FromRequestParts<S> for MailgunSigningKey
where
    HmacKey: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        _parts: &mut http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(HmacKey::from_ref(state)))
    }
}
