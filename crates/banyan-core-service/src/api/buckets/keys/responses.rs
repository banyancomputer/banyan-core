use serde::Serialize;

#[derive(Serialize)]
pub struct CreateBucketKey {
    pub id: String,
    pub approved: bool,
    pub fingerprint: String,
}

#[derive(Serialize)]
pub struct DeleteBucketKey {
    pub id: String,
    pub approved: bool,
}
