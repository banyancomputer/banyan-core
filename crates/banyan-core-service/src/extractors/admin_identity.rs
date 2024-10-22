use axum::async_trait;
use axum::extract::{FromRef, FromRequestParts};
use http::request::Parts;
use uuid::Uuid;

use crate::database::Database;
use crate::extractors::session_identity::SessionIdentityError;
use crate::extractors::SessionIdentity;

pub const ADMIN_USERS: [&str; 9] = [
    "sam.stelfox@banyan.computer",
    "vera@banyan.computer",
    "sam@banyan.computer",
    "plamen@banyan.computer",
    "olive@banyan.computer",
    "vladyslav@boostylabs.com",
    "jason@banyan.computer",
    "naftuli@banyan.computer",
    "dana@banyan.computer",
];

/// Extracted identity from a request made with a server-signed JWT
pub struct AdminIdentity {
    identity: SessionIdentity,
}

impl AdminIdentity {
    pub fn new(identity: SessionIdentity) -> Self {
        Self { identity }
    }
}

impl AdminIdentity {
    pub fn email(&self) -> &str {
        self.identity.email()
    }

    pub fn session_id(&self) -> Uuid {
        self.identity.session_id()
    }

    pub fn user_id(&self) -> Uuid {
        self.identity.user_id()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AdminIdentity
where
    Database: FromRef<S>,
    SessionIdentity: FromRequestParts<S, Rejection = SessionIdentityError>,
    S: Send + Sync,
{
    type Rejection = SessionIdentityError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let identity = SessionIdentity::from_request_parts(parts, state).await?;

        if !ADMIN_USERS.contains(&identity.email()) {
            return Err(SessionIdentityError::NotAdmin(identity.email().to_string()));
        }

        Ok(AdminIdentity::new(identity))
    }
}
