use std::ops::Deref;
use std::sync::Arc;

use jwt_simple::prelude::*;

#[derive(Clone)]
pub struct ServiceVerificationKey(Arc<ES384PublicKey>);

impl ServiceVerificationKey {
    pub fn new(key: ES384PublicKey) -> Self {
        Self(Arc::new(key))
    }
}

impl Deref for ServiceVerificationKey {
    type Target = Arc<ES384PublicKey>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
