use serde::Serialize;

#[derive(Serialize)]
pub struct CreateBucketKey {
    pub id: String,
    pub approved: bool,
    pub fingerprint: String,
}

#[derive(Debug, Serialize)]
pub struct ReadBucketKey {
    pub id: String,
    pub approved: bool,
    pub pem: String,
    pub fingerprint: String,
}

#[derive(Debug, Serialize)]
pub struct ReadAllBucketKeys(pub(crate) Vec<ReadBucketKey>);

#[derive(Serialize)]
pub struct DeleteBucketKey {
    pub id: String,
    pub approved: bool,
}
