use time::OffsetDateTime;

#[derive(sqlx::FromRow)]
pub struct Snapshot {
    pub id: String,
    pub metadata_id: String,
    pub size: Option<i64>,
    pub created_at: OffsetDateTime,
}
