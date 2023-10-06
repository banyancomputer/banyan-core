use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use time::OffsetDateTime;

/// DeviceApiKey - used to authenticate API requests from devices
#[derive(Debug, FromRow)]
pub struct DeviceApiKey {
    pub id: String,
    pub account_id: String,
    pub fingerprint: String,
    pub pem: String,
}

// TODO: should we add the account_id to this?
/// BucketKey - data associated with a bucket key
#[derive(Serialize, Debug)]
pub struct BucketKey {
    pub id: String,
    pub bucket_id: String,
    pub approved: bool,
    pub pem: String,
    pub fingerprint: String,
}

impl From<String> for MetadataState {
    fn from(s: String) -> Self {
        match s.as_str() {
            "uploading" => MetadataState::Uploading,
            "upload_failed" => MetadataState::UploadFailed,
            "pending" => MetadataState::Pending,
            "current" => MetadataState::Current,
            "outdated" => MetadataState::Outdated,
            "deleted" => MetadataState::Deleted,
            _ => panic!("invalid bucket metadata state: {}", s),
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

    pub expected_data_size: i64,
    pub data_size: i64,
    pub state: MetadataState,

    // These are only set on successful uploads
    pub metadata_size: Option<i64>,
    pub metadata_hash: Option<String>,

    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

/// Bucket Metadata with Snapshot - data associated with the metadata for a bucket
#[derive(Debug, Serialize, FromRow)]
pub struct MetadataWithSnapshot {
    pub metadata: Metadata,
    pub snapshot_id: Option<String>,
}

/// Snapshot of a piece of metadata
#[derive(Debug, Serialize, FromRow)]
pub struct CreateSnapshot {
    pub id: String,
    pub created_at: OffsetDateTime,
}

/// Snapshot of a piece of metadata
#[derive(Debug, Serialize, FromRow)]
pub struct Snapshot {
    pub id: String,
    pub metadata_id: String,
    pub size: i64,
    pub created_at: OffsetDateTime,
}

/// Storage Host
#[derive(Debug, Serialize, FromRow)]
pub struct StorageHost {
    pub id: String,
    pub name: String,
    pub url: String,
    pub used_storage: i64,
    pub available_storage: i64,
    pub fingerprint: String,
    pub pem: String,
}
