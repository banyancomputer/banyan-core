use serde::Deserialize;
use validify::Validify;

#[derive(Clone, Debug, Deserialize, Validify)]
pub struct CreateBucketKey {
    // TODO: validate that this is a pem format
    // #[validify(custom = "crate::validators::pem")]
    pub pem: String,
}
