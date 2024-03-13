use serde::Serialize;

use crate::database::models::Snapshot;

#[derive(Serialize)]
pub struct ApiSnapshot {
    pub id: String,
    pub bucket_id: String,
    pub metadata_id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
    pub state: String,

    pub created_at: i64,
}

impl From<Snapshot> for ApiSnapshot {
    fn from(val: Snapshot) -> Self {
        Self {
            id: val.id,
            bucket_id: val.bucket_id,
            metadata_id: val.metadata_id,

            size: val.size,
            state: val.state.to_string(),

            created_at: val.created_at.unix_timestamp(),
        }
    }
}
