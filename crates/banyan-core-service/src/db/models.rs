use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};

/// Catch all get id of created resource
#[derive(Debug, FromRow)]
pub struct CreatedResource {
    pub id: String,
}

/// DeviceApiKey - used to authenticate API requests from devices
#[derive(Debug, FromRow)]
pub struct DeviceApiKey {
    pub id: String,
    pub account_id: String,
    pub fingerprint: String,
    pub pem: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(rename_all = "lowercase")]
/// Bucket type
pub enum BucketType {
    Backup,
    Interactive,
}

impl From<String> for BucketType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "backup" => BucketType::Backup,
            "interactive" => BucketType::Interactive,
            _ => panic!("invalid bucket type"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(rename_all = "lowercase")]
/// Storage Class
pub enum StorageClass {
    Hot,
    Warm,
    Cold,
}

impl From<String> for StorageClass {
    fn from(s: String) -> Self {
        match s.as_str() {
            "hot" => StorageClass::Hot,
            "warm" => StorageClass::Warm,
            "cold" => StorageClass::Cold,
            _ => panic!("invalid storage class"),
        }
    }
}

/// Bucket - data associated with a bucket
#[derive(Debug, FromRow)]
pub struct Bucket {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub r#type: BucketType,
    pub storage_class: StorageClass,
}

// TODO: should we add the account_id to this?
/// BucketKey - data associated with a bucket key
#[derive(Serialize)]
pub struct BucketKey {
    pub id: String,
    pub bucket_id: String,
    pub approved: bool,
    pub pem: String,
}

/// MetadataState - state of metadata for a bucket
#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum MetadataState {
    Uploading,
    UploadFailed,
    Pending,
    Current,
    Outdated,
    Deleted,
}

impl Display for MetadataState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MetadataState::Uploading => write!(f, "uploading"),
            MetadataState::UploadFailed => write!(f, "upload_failed"),
            MetadataState::Pending => write!(f, "pending"),
            MetadataState::Current => write!(f, "current"),
            MetadataState::Outdated => write!(f, "outdated"),
            MetadataState::Deleted => write!(f, "deleted"),
        }
    }
}

impl From<String> for MetadataState {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Uploading" => MetadataState::Uploading,
            "pload_failed" => MetadataState::UploadFailed,
            "pending" => MetadataState::Pending,
            "current" => MetadataState::Current,
            "outdated" => MetadataState::Outdated,
            "deleted" => MetadataState::Deleted,
            _ => panic!("invalid bucket metadata state"),
        }
    }
}

/// Bucket Metadata - data associated with the metadata for a bucket
#[derive(Debug, Serialize, FromRow)]
pub struct Metadata {
    pub id: String,
    pub bucket_id: String,

    pub root_cid: String,
    pub metadata_cid: String,

    pub data_size: i64,
    pub state: MetadataState,

    // These are only set on successful uploads
    pub metadata_size: Option<i64>,
    pub metadata_hash: Option<String>,

    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

/// Snapshot of a piece of metadata
#[derive(Debug, Serialize, FromRow)]
pub struct Snapshot {
    pub id: String,
    pub metadata_id: String,
    pub created_at: chrono::NaiveDateTime,
}

/// Storage Host
#[derive(Debug, Serialize, FromRow)]
pub struct StorageHost {
    pub id: String,
    pub name: String,
    pub url: String,
    pub available_storage: i64,
    pub pem: String,
}
