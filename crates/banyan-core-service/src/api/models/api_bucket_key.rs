use serde::Serialize;

use crate::database::models::{ApiKey, BucketAccessState};

#[derive(Serialize)]
pub struct ApiBucketAccess {
    pub state: BucketAccessState,
    pub pem: String,
    pub fingerprint: String,
}

impl From<ApiKey> for ApiBucketAccess {
    fn from(val: ApiKey) -> Self {
        Self {
            id: val.id,
            state: val.state,
            pem: val.pem,
            fingerprint: val.fingerprint,
        }
    }
}
