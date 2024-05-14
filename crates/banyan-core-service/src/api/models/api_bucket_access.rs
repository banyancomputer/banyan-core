use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiBucketAccess {
    pub user_key_id: String,
    pub bucket_id: String,
    pub fingerprint: String,
    pub approved: bool,
}
