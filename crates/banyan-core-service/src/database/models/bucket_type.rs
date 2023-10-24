use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(rename_all = "lowercase")] // todo: make snake_case, requires db changes
pub enum BucketType {
    Backup,
    Interactive,
}

// todo: should be tryfrom since this is fallible
impl From<String> for BucketType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "backup" => BucketType::Backup,
            "interactive" => BucketType::Interactive,
            _ => panic!("invalid bucket type"),
        }
    }
}
