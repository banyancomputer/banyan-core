use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, sqlx::Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[sqlx(rename_all = "lowercase")] // todo: make snake_case, requires db changes
pub enum StorageClass {
    Hot,
    Warm,
    Cold,
}

// todo: should be try_from since this is fallible
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
