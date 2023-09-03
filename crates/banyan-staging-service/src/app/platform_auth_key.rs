use std::ops::Deref;
use std::sync::Arc;

use axum::{async_trait, Json, RequestPartsExt};
use axum::extract::{FromRef, FromRequestParts, TypedHeader};
use axum::http::request::Parts;

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

#[async_trait]
impl<S> FromRequestParts<S> for PlatformAuthKey
where
    PlatformAuthKey: FromRef<S>,
    S: Sync,
{
    type Rejection = ();

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(PlatformAuthKey::from_ref(state))
    }
}
