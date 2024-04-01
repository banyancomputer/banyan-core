use serde::Serialize;

use crate::database::models::BucketKey;

#[derive(Serialize)]
pub struct ApiBucketKey {
    pub id: String,
    pub bucket_id: String,
    pub approved: bool,
    #[serde(rename = "pem")]
    pub public_key: String,
    pub fingerprint: String,
}

impl From<BucketKey> for ApiBucketKey {
    fn from(val: BucketKey) -> Self {
        Self {
            id: val.id,
            bucket_id: val.bucket_id,
            approved: val.approved,
            public_key: val.pem,
            fingerprint: val.fingerprint,
        }
    }
}
