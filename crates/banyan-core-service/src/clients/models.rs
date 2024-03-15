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
    pub storage_grant_id: String,
    pub storage_grant_size: i64,
    pub block_cids: Vec<String>,
    pub new_host_id: String,
    pub new_host_url: String,
}

#[derive(Serialize)]
pub struct ReplicateDataRequest {
    pub metadata_id: String,
    pub block_cids: Vec<String>,
    pub new_host_id: String,
    pub new_host_url: String,
    pub old_host_id: String,
    pub old_host_url: String,
}



#[derive(Serialize)]
pub struct DeleteBlocksRequest {
    pub normalized_cids: Vec<String>,
    pub metadata_id: String,
    pub reset_storage_grant: Option<GrantResetRequest>,
}
#[derive(Serialize)]
pub struct GrantResetRequest {
    pub old_grant_id: String,
    pub new_grant_id: String,
    pub new_grant_size: i64,
}
