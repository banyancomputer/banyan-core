use std::ops::Deref;
use std::sync::Arc;

use ring::hmac::{HMAC_SHA256, Key};

#[derive(Clone)]
pub struct MailgunSigningKey(Arc<Key>);

impl MailgunSigningKey {
    pub fn new(key: &str) -> Self {
        let hmac_key = Key::new(HMAC_SHA256, key.as_bytes());
        Self(Arc::new(hmac_key))
    }
}

impl Deref for MailgunSigningKey {
    type Target = Arc<Key>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
