use std::sync::Arc;

use oauth2::{ClientId, ClientSecret};

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
