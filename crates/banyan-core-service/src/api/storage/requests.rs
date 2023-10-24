use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct FinalizeUpload {
    pub data_size: u64,
    pub normalized_cids: Vec<String>,
}
