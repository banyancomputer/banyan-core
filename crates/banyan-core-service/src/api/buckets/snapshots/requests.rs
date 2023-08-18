use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
pub struct CreateSnapshotRequest {
    pub metadata_id: String,
} 
