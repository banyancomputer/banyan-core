use serde::Serialize;

use crate::database::models::MetadataState;

#[derive(Serialize)]
pub struct ApiMetadata {
    pub id: String,

    pub root_cid: String,
    pub metadata_cid: String,
    pub data_size: i64,

    pub state: MetadataState,

    pub created_at: i64,
    pub updated_at: i64,

    pub snapshot_id: Option<String>,
}
