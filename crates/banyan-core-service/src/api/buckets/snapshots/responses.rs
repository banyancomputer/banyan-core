use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct CreateSnapshotResponse {
    pub id: String,
    pub created_at: i64,
}

#[derive(Debug, Serialize)]
pub struct ReadSnapshotResponse {
    pub id: String,
    pub metadata_id: String,
    pub created_at: i64,
} 

#[derive(Debug, Serialize)]
pub struct ReadAllSnapshotsResponse(pub Vec<ReadSnapshotResponse>);

#[derive(Debug, Serialize)]
pub struct RestoreSnapshotResponse {
    pub metadata_id: String,
}
