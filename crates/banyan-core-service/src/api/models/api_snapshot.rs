use crate::database::models::Snapshot;

pub struct ApiSnapshot {
    pub id: String,
    pub metadata_id: String,
    pub size: i64,
    pub created_at: i64,
}

impl From<Snapshot> for ApiSnapshot {
    fn from(val: Snapshot) -> Self {
        Self {
            id: val.id,
            metadata_id: val.metadata_id,
            size: val.size,
            created_at: val.created_at.timestamp(),
        }
    }
}
