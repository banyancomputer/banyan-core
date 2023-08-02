use serde::Deserialize;
use validify::Validify;

use crate::api::buckets::responses::BucketType;

#[derive(Clone, Debug, Deserialize, Validify)]
pub struct CreateBucket {
    #[validate(length(min = 3, max = 32))]
    pub friendly_name: String,
    pub r#type: BucketType,
    pub pem: String,
}
