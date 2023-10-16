use chrono::NaiveDateTime;

#[derive(sqlx::FromRow)]
pub struct Snapshot {
    pub id: String,
    pub metadata_id: String,
    pub size: i64,
    pub created_at: NaiveDateTime,
}
