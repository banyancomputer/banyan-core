use std::ops::Deref;

use axum::extract::FromRef;
use url::Url;

use crate::database::Database;
use crate::utils::VerificationKey;

use super::{AppState, Secrets};

// Helper struct for extracting state from requests
pub struct ServiceName(String);
pub struct ServiceHostname(Url);
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
        ServiceVerificationKey(state.service_verification_key().clone())
    }
}

impl FromRef<AppState> for PlatformName {
    fn from_ref(state: &AppState) -> Self {
        PlatformName(state.platform_name().to_string())
    }
}

impl FromRef<AppState> for PlatformHostname {
    fn from_ref(state: &AppState) -> Self {
        PlatformHostname(state.platform_hostname().clone())
    }
}

impl FromRef<AppState> for PlatformVerificationKey {
    fn from_ref(state: &AppState) -> Self {
        PlatformVerificationKey(state.platform_verification_key().clone())
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

impl Deref for ServiceVerificationKey {
    type Target = VerificationKey;

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
