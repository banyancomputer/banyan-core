use serde::Serialize;

use crate::database::models::{Bucket, BucketType, StorageClass};

#[derive(Serialize)]
pub struct ApiBucket {
    pub id: String,
    pub name: String,
    pub r#type: BucketType,
    pub storage_class: StorageClass,
}

impl From<Bucket> for ApiBucket {
    fn from(value: Bucket) -> Self {
        Self {
            id: value.id,
            name: value.name,
            r#type: value.r#type,
            storage_class: value.storage_class,
        }
    }
}
