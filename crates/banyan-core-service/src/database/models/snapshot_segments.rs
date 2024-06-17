use time::OffsetDateTime;

#[allow(dead_code)]
#[derive(sqlx::FromRow, Clone, Debug)]
pub struct SnapshotSegment {
    pub id: String,
    pub deal_id: String,
    pub size: i64,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}
