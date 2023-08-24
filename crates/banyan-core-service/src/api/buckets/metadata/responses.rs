use crate::db::models::MetadataState;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PushMetadataResponse {
    pub id: String,
    pub state: MetadataState,
    pub storage_host: Option<String>,
    pub storage_authorization: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReadMetadataResponse {
    pub id: String,
    pub root_cid: String,
    pub metadata_cid: String,
    pub data_size: i64,
    pub state: MetadataState,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize)]
pub struct ReadAllMetadataResponse(pub Vec<ReadMetadataResponse>);
