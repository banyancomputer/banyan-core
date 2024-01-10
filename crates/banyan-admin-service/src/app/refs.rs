use std::ops::Deref;
use std::sync::Arc;

use axum::extract::FromRef;
use jwt_simple::algorithms::ES384PublicKey;
use url::Url;

use super::{AppState, Secrets};
use crate::database::Database;
use crate::utils::VerificationKey;

// Helper struct for extracting state from requests
pub struct ServiceName(String);

pub struct ServiceHostname(Url);

#[derive(Clone)]
pub struct ServiceVerificationKey(VerificationKey);

pub struct PlatformName(String);

pub struct PlatformHostname(Url);

pub struct PlatformVerificationKey(VerificationKey);

impl FromRef<AppState> for Database {
    fn from_ref(state: &AppState) -> Self {
        state.database()
    }
}

impl FromRef<AppState> for Secrets {
    fn from_ref(state: &AppState) -> Self {
        state.secrets()
    }
}

impl FromRef<AppState> for ServiceName {
    fn from_ref(state: &AppState) -> Self {
        ServiceName(state.service_name().to_string())
    }
}

impl FromRef<AppState> for ServiceHostname {
    fn from_ref(state: &AppState) -> Self {
        ServiceHostname(state.service_hostname().clone())
    }
}

impl FromRef<AppState> for ServiceVerificationKey {
    fn from_ref(state: &AppState) -> Self {
        state.service_verification_key().clone()
    }
}

impl ServiceVerificationKey {
    pub fn new(key: ES384PublicKey) -> Self {
        Self(VerificationKey(Arc::new(key)))
    }
}

impl Deref for ServiceVerificationKey {
    type Target = Arc<ES384PublicKey>;

    fn deref(&self) -> &Self::Target {
        &self.0
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

impl Deref for PlatformName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for PlatformHostname {
    type Target = Url;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for PlatformVerificationKey {
    type Target = VerificationKey;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
