use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct FinalizeUpload {
    pub data_size: u64,
}
