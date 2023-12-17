use std::fmt;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotState {
    Pending,
    Completed,
}

impl From<String> for SnapshotState {
    fn from(s: String) -> Self {
        match s.as_str() {
            "pending" => SnapshotState::Pending,
            "completed" => SnapshotState::Completed,
            _ => panic!("invalid bucket type"),
        }
    }
}
impl Display for SnapshotState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SnapshotState::Pending => f.write_str("pending"),
            SnapshotState::Completed => f.write_str("completed"),
        }
    }
}
