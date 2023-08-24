use std::ops::Deref;
use std::sync::Arc;

use jwt_simple::prelude::*;

#[derive(Clone)]
pub struct GrantVerificationKey(Arc<ES384PublicKey>);

impl GrantVerificationKey {
    pub fn new(key: ES384PublicKey) -> Self {
        Self(Arc::new(key))
    }
}

impl Deref for GrantVerificationKey {
    type Target = Arc<ES384PublicKey>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
