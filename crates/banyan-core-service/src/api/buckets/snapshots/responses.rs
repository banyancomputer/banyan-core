use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CreateSnapshotResponse {
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct ReadSnapshotResponse {
    pub id: String,
    pub metadata_id: String,
    pub deal_id: String,
    pub root_cid: String,
    pub metadata_cid: String,
    pub data_size: i64,
}

#[derive(Debug, Serialize)]
pub struct ReadAllSnapshotsResponse(pub Vec<ReadSnapshotResponse>);
