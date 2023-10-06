use crate::database::models::{BucketType, StorageClass};

#[derive(Debug, sqlx::FromRow)]
pub struct Bucket {
    pub id: String,
    pub account_id: String,

    pub name: String,
    pub r#type: BucketType,
    pub storage_class: StorageClass,
}
