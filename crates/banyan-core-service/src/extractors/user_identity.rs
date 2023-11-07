use axum_extra::either::Either;

use super::{ApiIdentity, SessionIdentity};

pub type UserIdentity = Either<SessionIdentity, ApiIdentity>;

pub trait Identity {
    /// Returns the user id of the identity.
    fn user_id(&self) -> String;
    /// Return the key id of authentication key used to create the identity.
    fn key_id(&self) -> String;
}

impl Identity for UserIdentity {
    fn user_id(&self) -> String {
        match self {
            Either::E1(session) => session.user_id().to_string(),
            Either::E2(api) => api.user_id.clone(),
        }
    }
    fn key_id(&self) -> String {
        match self {
            Either::E1(session) => session.key_id().to_string(),
            Either::E2(api) => api.device_api_key_fingerprint.clone(),
        }
    }
}
