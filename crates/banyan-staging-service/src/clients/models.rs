use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct MeterTrafficRequest<'a> {
    pub user_id: &'a str,
    pub storage_host_name: &'a str,
    pub ingress: i64,
    pub egress: i64,
    pub slot: i64,
}
