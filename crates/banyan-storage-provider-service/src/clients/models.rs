use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub struct MeterTrafficRequest<'a> {
    pub user_id: &'a str,
    pub ingress: i64,
    pub egress: i64,
    pub slot: i64,
}

#[derive(Serialize)]
pub struct ReportRedistributionRequest {
    pub replication: bool,
    pub data_size: i64,
    pub normalized_cids: Vec<String>,
    pub grant_id: String,
}

#[derive(Serialize)]
pub struct ReportUploadRequest {
    pub data_size: u64,
    pub normalized_cids: Vec<String>,
    pub storage_authorization_id: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct ApiDeal {
    pub id: String,
    pub state: String,
    pub size: i64,
    pub accepted_by: Option<String>,
    pub accepted_at: Option<String>,
}
