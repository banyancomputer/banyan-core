use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
pub struct PushBucketMetadataRequest {
    /// The amount of data in bytes that the user wants to store in the bucket.
    /// This is NOT the size of the metadata file.
    pub data_size: usize,
    /// The CID of the metadata file.
    pub metadata_cid: String,
    /// The CID of the data the user wants to store in the bucket.
    pub root_cid: String,
}
