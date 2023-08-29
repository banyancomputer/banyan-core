use serde::Serialize;

#[derive(Serialize)]
pub struct CreateBucketKey {
    pub id: String,
    pub approved: bool,
}

#[derive(Debug, Serialize)]
pub struct ReadBucketKey {
    pub id: String,
    pub approved: bool,
    pub pem: String,
}

#[derive(Debug, Serialize)]
pub struct ReadAllBucketKeys(pub(crate) Vec<ReadBucketKey>);

#[derive(Serialize)]
pub struct DeleteBucketKey {
    pub id: String,
    pub approved: bool,
}

#[derive(Serialize)]
pub struct ApproveBucketKey {
    pub id: String,
    pub approved: bool,
    pub pem: String,
}
