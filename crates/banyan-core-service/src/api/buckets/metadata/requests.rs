use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
pub struct PushMetadataRequest {
    // TODO: Do we need to validate this? Especially to see if its negative?
    /// The amount of data in bytes that the user wants to store in the bucket.
    /// This is NOT the size of the metadata file.
    pub expected_data_size: usize,
    /// The CID of the data the user wants to store in the bucket.
    pub root_cid: String,
    /// Fingerprints of Public Keys which are being associated with a Bucket
    pub valid_keys: Vec<String>
}
