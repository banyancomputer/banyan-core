#[derive(sqlx::FromRow)]
pub struct UserKeyAccess {
    pub id: String,
    pub user_id: String,
    pub pem: String,
    pub fingerprint: String,
    pub bucket_ids: String,
}
