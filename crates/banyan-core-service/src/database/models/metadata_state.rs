use std::fmt::{self, Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, sqlx::Type)]
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
