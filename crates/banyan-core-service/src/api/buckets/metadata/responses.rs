use crate::db::models::BucketMetadataState;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PushBucketMetadataResponse {
    pub id: String,
    pub state: BucketMetadataState,
    pub storage_host: String,
    pub storage_authorization: String,
}

#[derive(Debug, Serialize)]
pub struct ReadBucketMetadataResponse {
    pub id: String,
    pub root_cid: String,
    pub metadata_cid: String,
    pub data_size: i64,
    pub state: BucketMetadataState,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize)]
pub struct ReadAllBucketMetadataResponse(pub Vec<ReadBucketMetadataResponse>);
