use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use uuid::Uuid;

use crate::extractors::api_identity::{ApiIdentity, ApiIdentityError};
use crate::extractors::session_identity::{SessionIdentity, SessionIdentityError};

pub enum UserIdentity {
    Api(ApiIdentity),
    Session(SessionIdentity),
}

impl UserIdentity {
    pub fn id(&self) -> Uuid {
        match &self {
            UserIdentity::Api(api) => api.user_id(),
            UserIdentity::Session(session) => session.user_id(),
        }
    }

    pub fn key_fingerprint(&self) -> &str {
        match &self {
            UserIdentity::Api(api) => api.key_fingerprint(),
            UserIdentity::Session(session) => session.key_fingerprint(),
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
    type Rejection = ApiIdentityError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        if let Ok(session) = SessionIdentity::from_request_parts(parts, state).await {
            return Ok(UserIdentity::Session(session));
        }

        let api = ApiIdentity::from_request_parts(parts, state).await?;
        Ok(UserIdentity::Api(api))
    }
}
