use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(rename_all = "lowercase")]
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
