use std::collections::BTreeMap;
use std::convert::Infallible;
use std::sync::Arc;

use axum::async_trait;
use axum::extract::{FromRef, FromRequestParts};
use http::request::Parts;

mod mailgun_signing_key;
mod provider_credential;
mod service_signing_key;

pub use mailgun_signing_key::MailgunSigningKey;
pub use provider_credential::ProviderCredential;
pub use service_signing_key::ServiceSigningKey;

use crate::app::AppState;

#[derive(Clone)]
pub struct Secrets {
    mailgun_signing_key: Option<MailgunSigningKey>,
    provider_credentials: Arc<BTreeMap<Arc<str>, ProviderCredential>>,
    service_signing_key: ServiceSigningKey,
}

impl Secrets {
    pub fn mailgun_signing_key(&self) -> Option<MailgunSigningKey> {
        self.mailgun_signing_key.clone()
    }

    pub fn new(
        credentials: BTreeMap<Arc<str>, ProviderCredential>,
        mailgun_signing_key: Option<MailgunSigningKey>,
        service_signing_key: ServiceSigningKey,
    ) -> Self {
        Self {
            mailgun_signing_key,
            provider_credentials: Arc::new(credentials),
            service_signing_key,
        }
    }

    pub fn provider_credential(&self, config_id: &str) -> Option<&ProviderCredential> {
        self.provider_credentials.get(config_id)
    }

    pub fn service_signing_key(&self) -> ServiceSigningKey {
        self.service_signing_key.clone()
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
