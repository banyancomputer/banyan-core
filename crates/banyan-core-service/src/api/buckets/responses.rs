use crate::api::buckets::keys;
use crate::db::models::*;
use serde::Serialize;
use sqlx::FromRow;

#[derive(Serialize)]
pub struct CreateBucket {
    pub id: String,
    pub name: String,
    pub r#type: BucketType,
    pub storage_class: StorageClass,
    pub initial_bucket_key: keys::responses::CreateBucketKey,
}

#[derive(Serialize, FromRow)]
pub struct ReadBucket {
    pub id: String,
    pub name: String,
    pub r#type: BucketType,
    pub storage_class: StorageClass,
}

#[derive(Serialize)]
pub struct ReadBuckets(pub Vec<ReadBucket>);

#[derive(Serialize)]
pub struct DeleteBucket {
    pub id: String,
    pub name: String,
}

#[derive(Serialize)]
pub struct GetUsage {
    pub size: i64,
}
