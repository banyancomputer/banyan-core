#[derive(Debug, sqlx::FromRow)]
pub struct BucketKey {
    pub id: String,
    pub bucket_id: String,

    pub approved: bool,
    pub pem: String,
    pub fingerprint: String,
}
