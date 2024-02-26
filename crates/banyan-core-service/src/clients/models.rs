use serde::Serialize;

#[derive(Serialize)]
pub struct ClientDataRequest {
    pub fingerprint: String,
    pub public_key: String,
    pub platform_id: String,
}

#[derive(Serialize)]
pub struct DistributeDataRequest {
    pub metadata_id: String,
    pub grant_id: String,
    pub block_cids: Vec<String>,
    pub new_host_id: String,
    pub new_host_url: String,
}

#[derive(Serialize)]
pub struct DeleteBlocksRequest {
    pub normalized_cids: Vec<String>,
    pub metadata_id: String,
}
