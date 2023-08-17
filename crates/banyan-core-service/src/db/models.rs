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

/// Bucket - data associated with a bucket
#[derive(Debug, FromRow)]
pub struct Bucket {
    pub id: String,
    pub account_id: String,
    pub name: String,
    pub r#type: BucketType,
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

/// BucketMetadataState - state of metadata for a bucket
#[derive(Debug, Serialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum BucketMetadataState {
    Uploading,
    UploadFailed,
    Pending,
    Current,
    Outdated,
    Deleted,
}

impl Display for BucketMetadataState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BucketMetadataState::Uploading => write!(f, "uploading"),
            BucketMetadataState::UploadFailed => write!(f, "upload_failed"),
            BucketMetadataState::Pending => write!(f, "pending"),
            BucketMetadataState::Current => write!(f, "current"),
            BucketMetadataState::Outdated => write!(f, "outdated"),
            BucketMetadataState::Deleted => write!(f, "deleted"),
        }
    }
}

impl From<String> for BucketMetadataState {
    fn from(s: String) -> Self {
        println!("s: {}", s);
        match s.as_str() {
            "Uploading" => BucketMetadataState::Uploading,
            "pload_failed" => BucketMetadataState::UploadFailed,
            "pending" => BucketMetadataState::Pending,
            "current" => BucketMetadataState::Current,
            "outdated" => BucketMetadataState::Outdated,
            "deleted" => BucketMetadataState::Deleted,
            _ => panic!("invalid bucket metadata state"),
        }
    }
}

/// Bucket Metadata - data associated with the metadata for a bucket
#[derive(Debug, Serialize, FromRow)]
pub struct BucketMetadata {
    pub id: String,
    pub bucket_id: String,

    pub root_cid: String,
    pub metadata_cid: String,

    pub data_size: i64,
    pub state: BucketMetadataState,

    // These are only set on successful uploads
    pub size: Option<i64>,
    pub hash: Option<String>,

    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}
