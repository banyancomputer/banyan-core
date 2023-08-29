use std::ops::Deref;
use std::sync::Arc;

use jwt_simple::prelude::*;

#[derive(Clone)]
pub struct PlatformAuthKey(Arc<ES384KeyPair>);

impl PlatformAuthKey {
    pub fn new(key: ES384KeyPair) -> Self {
        Self(Arc::new(key))
    }
}

impl Deref for PlatformAuthKey {
    type Target = Arc<ES384KeyPair>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
