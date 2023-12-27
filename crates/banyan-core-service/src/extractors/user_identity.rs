use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_extra::either::Either;
use uuid::Uuid;

use crate::extractors::api_identity::{ApiIdentity, ApiIdentityError};
use crate::extractors::session_identity::{SessionIdentity, SessionIdentityError};

/// Enum encompassing Authentication Strategies for API requests
pub struct UserIdentity(Either<ApiIdentity, SessionIdentity>);

impl UserIdentity {
    pub fn id(&self) -> Uuid {
        match &self.0 {
            Either::E1(api) => api.user_id(),
            Either::E2(session) => session.user_id(),
        }
    }

    pub fn key_fingerprint(&self) -> &str {
        match &self.0 {
            Either::E1(api) => api.key_fingerprint(),
            Either::E2(session) => session.key_fingerprint(),
        }
    }

    pub fn ticket_subject(&self) -> String {
        format!("{}@{}", self.id(), self.key_fingerprint())
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for UserIdentity
where
    SessionIdentity: FromRequestParts<S, Rejection = SessionIdentityError>,
    ApiIdentity: FromRequestParts<S, Rejection = ApiIdentityError>,
    S: Send + Sync,
{
    type Rejection = SessionIdentityError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let either: Either<ApiIdentity, SessionIdentity> =
            Either::from_request_parts(parts, state).await?;
        Ok(UserIdentity(either))
    }
}
