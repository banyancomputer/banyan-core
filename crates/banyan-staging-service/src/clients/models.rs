use cid::Cid;
use jwt_simple::prelude::Deserialize;
use serde::Serialize;

#[derive(Serialize)]
pub struct ClientsRequest {
    pub platform_id: String,
    pub fingerprint: String,
    pub public_key: String,
}

#[derive(Serialize)]
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
    pub details: BlockUploadDetailsRequest,
}

#[derive(Serialize)]
pub struct BlockUploadDetailsRequest {
    pub completed: bool,
    pub upload_id: String,
    pub grant_id: String,
}

#[derive(Deserialize)]
pub struct NewClientResponse {
    pub id: String,
}

#[derive(Serialize)]
pub struct NewUploadRequest {
    pub metadata_id: String,
    pub client_id: String,
    pub grant_size: i64,
    pub grant_id: String,
}
