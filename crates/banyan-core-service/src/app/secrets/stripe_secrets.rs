use std::sync::Arc;

#[derive(Clone)]
pub struct StripeSecrets {
    secret: Arc<str>,
    webhook_key: Arc<str>,
}

impl StripeSecrets {
    pub fn new(secret: String, webhook_key: String) -> Self {
        Self {
            secret: secret.into(),
            webhook_key: webhook_key.into(),
        }
    }

    pub fn secret(&self) -> &str {
        &self.secret
    }

    pub fn webhook_key(&self) -> &str {
        &self.webhook_key
    }
}
