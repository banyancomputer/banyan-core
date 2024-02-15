use cid::Cid;
use jwt_simple::prelude::Deserialize;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct MeterTrafficRequest<'a> {
    pub user_id: &'a str,
    pub ingress: i64,
    pub egress: i64,
    pub slot: i64,
}

#[derive(Deserialize)]
pub struct StorageProviderAuthResponse {
    pub token: String,
}

#[derive(Serialize)]
pub struct ReportUploadRequest {
    pub data_size: u64,
    pub normalized_cids: Vec<String>,
    pub storage_authorization_id: String,
}

#[derive(Deserialize)]
pub struct NewUploadResponse {
    pub upload_id: String,
}

#[derive(Serialize)]
pub struct BlockUploadRequest {
    pub cid: Cid,
    // Optional additional details about the nature of the upload
    #[serde(flatten)]
    pub details: BlockUploadDetailsRequest,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum BlockUploadDetailsRequest {
    Ongoing { completed: bool, upload_id: String },
    OneOff,
}
