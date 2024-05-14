use time::OffsetDateTime;

#[derive(sqlx::FromRow)]
pub struct UserKeyAccess {
    pub id: String,
    pub name: String,
    pub user_id: String,
    pub api_access: bool,
    pub public_key: String,
    pub fingerprint: String,
    pub created_at: OffsetDateTime,
    pub bucket_ids: String,
}
