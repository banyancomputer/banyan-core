use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Serialize)]
pub struct BucketAccess {
    user_key_id: String,
    bucket_id: String,
    state: BucketAccessState,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
pub enum BucketAccessState {
    Pending,
    Approved,
    Revoked,
}
