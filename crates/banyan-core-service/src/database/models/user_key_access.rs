#[derive(sqlx::FromRow)]
pub struct UserKeyAccess {
    pub id: String,
    pub name: String,
    pub user_id: String,
    pub api_access: bool,
    pub pem: String,
    pub fingerprint: String,
    pub bucket_ids: String,
}
