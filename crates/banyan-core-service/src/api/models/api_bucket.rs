use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::database::models::{Bucket, BucketType, StorageClass};

#[derive(Serialize, Deserialize)]
pub struct ApiBucket {
    pub id: String,
    pub owner_id: String,

    pub name: String,
    pub r#type: BucketType,
    pub storage_class: StorageClass,

    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}

impl From<Bucket> for ApiBucket {
    fn from(value: Bucket) -> Self {
        Self {
            id: value.id,
            owner_id: value.user_id,

            name: value.name,
            r#type: value.r#type,
            storage_class: value.storage_class,

            updated_at: value.updated_at,
        }
    }
}
