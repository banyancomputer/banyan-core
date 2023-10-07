use serde::Serialize;

use crate::database::models::{MetadataState, PartialMetadataWithSnapshot};

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

impl From<PartialMetadataWithSnapshot> for ApiMetadata {
    fn from(value: PartialMetadataWithSnapshot) -> Self {
        Self {
            id: value.id,

            root_cid: value.root_cid,
            metadata_cid: value.metadata_cid,
            data_size: value.data_size.unwrap_or(0),

            state: value.state,

            created_at: value.created_at.unix_timestamp(),
            updated_at: value.updated_at.unix_timestamp(),

            snapshot_id: value.snapshot_id,
        }
    }
}
