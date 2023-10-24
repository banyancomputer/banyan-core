use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, PartialEq, Serialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(rename_all = "snake_case")]
pub enum MetadataState {
    Uploading,
    UploadFailed,
    Pending,
    Current,
    Outdated,
    Deleted,
}

// todo: might not be needed...
impl Display for MetadataState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            MetadataState::Uploading => f.write_str("uploading"),
            MetadataState::UploadFailed => f.write_str("upload_failed"),
            MetadataState::Pending => f.write_str("pending"),
            MetadataState::Current => f.write_str("current"),
            MetadataState::Outdated => f.write_str("outdated"),
            MetadataState::Deleted => f.write_str("deleted"),
        }
    }
}

// todo: should be tryfrom since this is fallible... might not be needed at all
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
