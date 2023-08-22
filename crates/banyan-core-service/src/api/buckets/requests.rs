use serde::Deserialize;
use validify::Validify;

use crate::db::models::{BucketType, StorageClass};

#[derive(Clone, Debug, Deserialize, Validify)]
pub struct CreateBucket {
    #[validate(length(min = 3, max = 32))]
    pub name: String,
    pub r#type: BucketType,
    pub storage_class: StorageClass,
    pub initial_bucket_key_pem: String,
}
