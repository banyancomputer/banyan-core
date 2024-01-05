use crate::utils::SigningKey;

/// Application run-time secrets.rs
#[derive(Clone)]
pub struct Secrets {
    /// Application signing key
    service_signing_key: SigningKey,
}

impl Secrets {
    pub fn new(service_signing_key: SigningKey) -> Self {
        Self {
            service_signing_key,
        }
    }

    pub fn service_signing_key(&self) -> SigningKey {
        self.service_signing_key.clone()
    }
}
