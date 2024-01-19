use axum::async_trait;
use axum::extract::{FromRef, FromRequestParts};
use http::request::Parts;
use uuid::Uuid;

use crate::app::ServiceVerificationKey;
use crate::database::Database;
use crate::extractors::session_identity::SessionIdentityError;
use crate::extractors::SessionIdentity;

const ADMIN_USERS: [&str; 7] = [
    "sam.stelfox@banyan.computer",
    "alex@banyan.computer",
    "vera@banyan.computer",
    "sam@banyan.computer",
    "plamen@banyan.computer",
    "olive@banyan.computer",
    "vladyslav@boostylabs.com",
];

/// Extracted identity from a request made with a server-signed JWT
pub struct AdminIdentity {
    identity: SessionIdentity,
}

impl AdminIdentity {
    fn new(identity: SessionIdentity) -> Self {
        return Self { identity };
    }
}

impl AdminIdentity {
    pub fn session_id(&self) -> Uuid {
        self.identity.session_id()
    }

    pub fn user_id(&self) -> Uuid {
        self.identity.user_id()
    }

    pub fn key_fingerprint(&self) -> &str {
        self.identity.key_fingerprint()
    }
    pub fn email(&self) -> &str {
        self.identity.email()
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for AdminIdentity
where
    Database: FromRef<S>,
    ServiceVerificationKey: FromRef<S>,
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
