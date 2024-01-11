#[derive(Clone)]
pub struct StripeSecret(String);

impl StripeSecret {
    pub fn new(secret: String) -> Self {
        Self(secret)
    }
}
