use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct MeterTrafficRequest<'a> {
    pub user_id: &'a str,
    pub ingress: i64,
    pub egress: i64,
    pub slot: i64,
}

#[derive(Serialize)]
pub struct ReportRedistributionRequest {
    pub data_size: u64,
    pub normalized_cids: Vec<String>,
    pub grant_id: String,
}
