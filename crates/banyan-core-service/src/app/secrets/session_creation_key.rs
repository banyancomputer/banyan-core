use std::ops::Deref;
use std::sync::Arc;

use jwt_simple::prelude::*;

use crate::app::SessionVerificationKey;

#[derive(Clone)]
pub struct SessionCreationKey(Arc<ES384KeyPair>);

impl SessionCreationKey {
    pub fn new(key: ES384KeyPair) -> Self {
        Self(Arc::new(key))
    }
}

impl SessionCreationKey {
    pub fn verifier(&self) -> SessionVerificationKey {
        let key_pair = self.0.clone();
        SessionVerificationKey::new(key_pair.public_key())
    }
}

impl Deref for SessionCreationKey {
    type Target = Arc<ES384KeyPair>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
