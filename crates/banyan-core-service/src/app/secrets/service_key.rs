use std::ops::Deref;
use std::sync::Arc;

use jwt_simple::prelude::*;

use crate::app::ServiceVerificationKey;

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
