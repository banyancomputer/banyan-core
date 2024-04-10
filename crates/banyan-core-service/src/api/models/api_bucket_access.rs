use serde::{Deserialize, Serialize};

use crate::database::models::BucketAccessState;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiBucketAccess {
    pub user_key_id: String,
    pub bucket_id: String,
    pub fingerprint: String,
    pub state: BucketAccessState,
}
