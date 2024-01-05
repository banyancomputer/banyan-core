use std::collections::BTreeMap;
use std::convert::Infallible;
use std::ops::Deref;
use std::sync::Arc;

use axum::async_trait;
use axum::extract::{FromRef, FromRequestParts};
use http::request::Parts;
use jwt_simple::algorithms::ES384KeyPair;
use oauth2::{ClientId, ClientSecret};

use crate::app::refs::ServiceVerificationKey;
use crate::app::AppState;

#[derive(Clone)]
pub struct Secrets {
    provider_credentials: Arc<BTreeMap<Arc<str>, ProviderCredential>>,
    service_key: ServiceKey,
}

impl Secrets {
    pub fn new(
        credentials: BTreeMap<Arc<str>, ProviderCredential>,
        service_key: ServiceKey,
    ) -> Self {
        Self {
            provider_credentials: Arc::new(credentials),
            service_key,
        }
    }

    pub fn provider_credential(&self, config_id: &str) -> Option<&ProviderCredential> {
        self.provider_credentials.get(config_id)
    }

    pub fn service_key(&self) -> ServiceKey {
        self.service_key.clone()
    }
}

#[async_trait]
impl FromRequestParts<AppState> for Secrets {
    type Rejection = Infallible;

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Secrets::from_ref(state))
    }
}

#[derive(Clone)]
pub struct ProviderCredential {
    id: Arc<str>,
    secret: Arc<str>,
}

impl ProviderCredential {
    pub fn id(&self) -> ClientId {
        ClientId::new(self.id.to_string())
    }

    pub fn new(id: &str, secret: &str) -> Self {
        Self {
            id: Arc::from(id),
            secret: Arc::from(secret),
        }
    }

    pub fn secret(&self) -> ClientSecret {
        ClientSecret::new(self.secret.to_string())
    }
}

#[derive(Clone)]
pub struct ServiceKey(Arc<ES384KeyPair>);

impl ServiceKey {
    pub fn new(key: ES384KeyPair) -> Self {
        Self(Arc::new(key))
    }
}

impl ServiceKey {
    pub fn verifier(&self) -> ServiceVerificationKey {
        let key_pair = self.0.clone();
        ServiceVerificationKey::new(key_pair.public_key())
    }
}

impl Deref for ServiceKey {
    type Target = Arc<ES384KeyPair>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
