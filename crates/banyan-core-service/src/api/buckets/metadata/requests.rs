use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
pub struct PushMetadataRequest {
    /// The amount of data in bytes that the user wants to store in the bucket.
    /// This is NOT the size of the metadata file.
    pub expected_data_size: usize,
    /// The root CID of the content CAR
    pub root_cid: String,
    /// The root CID of the metadata CAR
    pub metadata_cid: String,
    /// Fingerprints of Public Keys which are being associated with a Bucket
    pub valid_keys: Vec<String>,
}
