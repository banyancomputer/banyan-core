use std::collections::BTreeMap;
use std::sync::Arc;

mod provider_credential;
mod session_creation_key;

pub(crate) use provider_credential::ProviderCredential;
pub(crate) use session_creation_key::SessionCreationKey;

#[derive(Clone)]
pub struct Secrets {
    provider_credentials: Arc<BTreeMap<Arc<str>, ProviderCredential>>,
    session_creation_key: SessionCreationKey,
}

impl Secrets {
    pub fn new(
        credentials: BTreeMap<Arc<str>, ProviderCredential>,
        session_creation_key: SessionCreationKey,
    ) -> Self {
        Self {
            provider_credentials: Arc::new(credentials),
            session_creation_key,
        }
    }

    pub fn provider_credential(&self, config_id: &str) -> Option<&ProviderCredential> {
        self.provider_credentials.get(config_id)
    }

    pub fn session_creation_key(&self) -> SessionCreationKey {
        self.session_creation_key.clone()
    }
}
