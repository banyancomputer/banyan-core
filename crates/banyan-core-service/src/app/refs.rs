use std::ops::Deref;
use std::sync::Arc;

use axum::extract::FromRef;
use jwt_simple::prelude::*;

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

impl FromRef<State> for ServiceVerificationKey {
    fn from_ref(state: &State) -> Self {
        state.service_verifier().clone()
    }
}
