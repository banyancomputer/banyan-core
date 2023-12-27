use crate::database::types::PrecisionTimestamp;

#[derive(sqlx::FromRow)]
pub struct Snapshot {
    pub id: String,
    pub metadata_id: String,
    pub state: String,
    pub size: Option<i64>,
    pub created_at: PrecisionTimestamp,
}
