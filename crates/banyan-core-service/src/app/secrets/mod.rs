use std::collections::BTreeMap;
use std::convert::Infallible;
use std::sync::Arc;

use axum::async_trait;
use axum::extract::{FromRef, FromRequestParts};
use http::request::Parts;

mod mailgun_signing_key;
mod provider_credential;
mod service_key;
mod stripe_secret;

pub use mailgun_signing_key::MailgunSigningKey;
pub use provider_credential::ProviderCredential;
pub use service_key::ServiceKey;
pub use stripe_secret::StripeSecret;

use crate::app::AppState;

#[derive(Clone)]
pub struct Secrets {
    mailgun_signing_key: Option<MailgunSigningKey>,
    provider_credentials: Arc<BTreeMap<Arc<str>, ProviderCredential>>,
    service_key: ServiceKey,
    stripe_secret: Option<StripeSecret>,
}

impl Secrets {
    pub fn mailgun_signing_key(&self) -> Option<MailgunSigningKey> {
        self.mailgun_signing_key.clone()
    }

    pub fn new(
        credentials: BTreeMap<Arc<str>, ProviderCredential>,
        mailgun_signing_key: Option<MailgunSigningKey>,
        service_key: ServiceKey,
        stripe_secret: Option<StripeSecret>,
    ) -> Self {
        Self {
            mailgun_signing_key,
            provider_credentials: Arc::new(credentials),
            service_key,
            stripe_secret,
        }
    }

    pub fn provider_credential(&self, config_id: &str) -> Option<&ProviderCredential> {
        self.provider_credentials.get(config_id)
    }

    pub fn service_key(&self) -> ServiceKey {
        self.service_key.clone()
    }

    pub fn stripe_secret(&self) -> Option<StripeSecret> {
        self.stripe_secret.clone()
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
