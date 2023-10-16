use serde::Serialize;

#[derive(Serialize)]
pub struct DeleteBucketKey {
    pub id: String,
    pub approved: bool,
}
