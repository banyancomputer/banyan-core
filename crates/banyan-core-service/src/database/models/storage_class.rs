use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};

#[derive(Clone, Debug, Deserialize, Serialize, Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(rename_all = "lowercase")]
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
