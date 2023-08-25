use std::ops::Deref;
use std::sync::Arc;

use jwt_simple::prelude::*;

#[derive(Clone)]
pub struct PlatformVerificationKey(Arc<ES384PublicKey>);

impl PlatformVerificationKey {
    pub fn new(key: ES384PublicKey) -> Self {
        Self(Arc::new(key))
    }
}

impl Deref for PlatformVerificationKey {
    type Target = Arc<ES384PublicKey>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
