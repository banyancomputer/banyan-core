use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(sqlx::FromRow)]
pub struct BucketKey {
    pub id: String,
    pub name: String,
    pub user_id: String,
    //pub bucket_id: String,
    pub api_access: bool,
    pub state: ApiKeyState,
    pub pem: String,
    pub fingerprint: String,

    pub updated_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
pub enum ApiKeyState {
    Pending,
    Approved,
    Revoked,
}
