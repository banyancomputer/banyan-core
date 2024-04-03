use crate::database::models::{BucketAccess, BucketAccessState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiBucketAccess {
    pub fingerprint: String,
    pub state: BucketAccessState,
}
