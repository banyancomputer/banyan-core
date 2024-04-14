use time::OffsetDateTime;

use crate::database::models::SnapshotState;

#[derive(Debug, sqlx::FromRow)]
pub struct Snapshot {
    pub id: String,
    pub bucket_id: String,
    pub metadata_id: String,
    pub state: SnapshotState,
    pub size: Option<i64>,
    pub tokens_used: i64,
    pub created_at: OffsetDateTime,
}
