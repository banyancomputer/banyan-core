use axum::async_trait;
use axum::extract::rejection::HostRejection;
use axum::extract::{FromRequestParts, Host};
use http::request::Parts;
use url::Url;

const X_FORWARDED_SCHEME_HEADER_KEY: &str = "X-Forwarded-Proto";

pub struct ServerBase(pub Url);

#[async_trait]
impl<S> FromRequestParts<S> for ServerBase
where
    S: Send + Sync,
{
    type Rejection = HostRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let connection_scheme = parts
            .headers
            .get(X_FORWARDED_SCHEME_HEADER_KEY)
            .and_then(|scheme| scheme.to_str().ok())
            .unwrap_or("http")
            .to_string();

        let host = Host::from_request_parts(parts, state).await?;

        let url = Url::parse(&format!("{connection_scheme}://{}", host.0))
            .expect("built value to be valid");

        Ok(ServerBase(url))
    }
}
