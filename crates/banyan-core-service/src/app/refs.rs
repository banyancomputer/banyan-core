use std::ops::Deref;
use std::sync::Arc;

use axum::extract::FromRef;
use jwt_simple::prelude::*;
use url::Url;

use crate::app::state::State;
use crate::app::Secrets;
use crate::database::Database;

#[derive(Clone)]
pub struct ServiceVerificationKey(Arc<ES384PublicKey>);

impl ServiceVerificationKey {
    pub fn new(key: ES384PublicKey) -> Self {
        Self(Arc::new(key))
    }
}

impl Deref for ServiceVerificationKey {
    type Target = Arc<ES384PublicKey>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Helper struct for extracting state from requests
pub struct ServiceName(String);
pub struct ServiceHostname(Url);

impl FromRef<State> for Database {
    fn from_ref(state: &State) -> Self {
        state.database()
    }
}

impl FromRef<State> for Secrets {
    fn from_ref(state: &State) -> Self {
        state.secrets()
    }
}

impl FromRef<State> for ServiceName {
    fn from_ref(state: &State) -> Self {
        ServiceName(state.service_name().to_string())
    }
}

impl FromRef<State> for ServiceVerificationKey {
    fn from_ref(state: &State) -> Self {
        state.service_verifier().clone()
    }
}

impl Deref for ServiceName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for ServiceHostname {
    type Target = Url;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
