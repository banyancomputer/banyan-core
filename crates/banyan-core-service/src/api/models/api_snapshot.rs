use serde::Serialize;

use crate::database::models::Snapshot;

#[derive(Serialize)]
pub struct ApiSnapshot {
    pub id: String,
    pub metadata_id: String,
    pub created_at: i64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
}

impl From<Snapshot> for ApiSnapshot {
    fn from(val: Snapshot) -> Self {
        Self {
            id: val.id,
            metadata_id: val.metadata_id,
            size: val.size,
            created_at: val.created_at.unix_timestamp(),
        }
    }
}
