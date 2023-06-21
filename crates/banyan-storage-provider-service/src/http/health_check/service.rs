use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::http::health_check::HealthCheckError;
use crate::http::health_check::HealthCheckResponse;

pub struct Service;

impl Service {
    pub async fn ready(&mut self) -> Result<(), HealthCheckError> {
        Ok(())
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for Service
where
    S: Send + Sync,
{
    type Rejection = HealthCheckResponse;

    async fn from_request_parts(_parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Service)
    }
}
