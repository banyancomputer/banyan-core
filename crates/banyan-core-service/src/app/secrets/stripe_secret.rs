use std::sync::Arc;

#[derive(Clone)]
pub struct StripeSecret(Arc<str>);

impl StripeSecret {
    pub fn new(secret: String) -> Self {
        Self(secret.into())
    }

    pub fn key(&self) -> &str {
        &self.0
    }
}
