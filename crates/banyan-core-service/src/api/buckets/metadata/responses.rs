use crate::db::models::MetadataState;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct PushMetadataResponse {
    pub id: String,
    pub state: MetadataState,
    pub storage_host: Option<String>,
    pub storage_authorization: Option<String>,
}
