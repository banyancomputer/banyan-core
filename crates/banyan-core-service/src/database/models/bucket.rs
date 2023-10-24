use crate::database::models::{BucketType, StorageClass};

#[derive(sqlx::FromRow)]
pub struct Bucket {
    pub id: String,

    // todo: should be associated to users table, requires a lot of rework
    pub account_id: String,

    pub name: String,
    pub r#type: BucketType,
    pub storage_class: StorageClass,
}
