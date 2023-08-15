use serde::Deserialize;
use validify::Validify;

#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
pub struct PushMetadataRequest {
    data_size: usize,
    metadata_cid: String,
    root_cid: String,
}
