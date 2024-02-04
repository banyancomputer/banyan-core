use sqlx::FromRow;
use time::OffsetDateTime;

#[derive(Debug, FromRow)]
pub struct StorageGrants {
    pub id: String,
    pub client_id: String,
    pub grant_id: String,
    pub allowed_storage: i64,
    pub created_at: OffsetDateTime,
}
