use serde::Serialize;

#[derive(Serialize)]
pub struct ClientData {
    pub fingerprint: String,
    pub public_key: String,
    pub platform_id: String,
}

#[derive(Serialize)]
pub struct UploadData {
    pub metadata_id: String,
    pub storage_host_id: String,
    pub storage_host_url: String,
}
#[derive(Serialize)]
pub struct PushDataRequest {
    pub clients: Vec<ClientData>,
    pub upload: UploadData,
}
