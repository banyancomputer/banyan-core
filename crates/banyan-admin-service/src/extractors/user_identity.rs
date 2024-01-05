use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use uuid::Uuid;

use crate::extractors::session_identity::{SessionIdentity, SessionIdentityError};

/// Enum encompassing Authentication Strategies for API requests
pub struct UserIdentity(SessionIdentity);

impl UserIdentity {
    pub fn id(&self) -> Uuid {
        self.0.user_id()
    }

    pub fn key_fingerprint(&self) -> &str {
        self.0.key_fingerprint()
    }

    pub fn ticket_subject(&self) -> String {
        format!("{}@{}", self.id(), self.key_fingerprint())
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for UserIdentity
where
    SessionIdentity: FromRequestParts<S, Rejection = SessionIdentityError>,
    S: Send + Sync,
{
    type Rejection = SessionIdentityError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let identity = SessionIdentity::from_request_parts(parts, state).await?;
        Ok(UserIdentity(identity))
    }
}
