use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(sqlx::FromRow, Serialize)]
pub struct ApiKey {
    pub id: String,
    pub name: String,
    pub user_id: String,

    pub api_access: bool,

    pub pem: String,
    pub fingerprint: String,

    //
    pub updated_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

#[derive(sqlx::FromRow, Serialize)]
struct BucketAccess {
    id: String,
    api_key_id: String,
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
