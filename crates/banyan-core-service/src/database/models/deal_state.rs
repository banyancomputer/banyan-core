use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
pub enum DealState {
    Active,
    Accepted,
    Sealed,
    Finalized,
    Cancelled,
}

impl From<String> for DealState {
    fn from(s: String) -> Self {
        match s.as_str() {
            "active" => DealState::Active,
            "accepted" => DealState::Accepted,
            "sealed" => DealState::Sealed,
            "finalized" => DealState::Finalized,
            "cancelled" => DealState::Cancelled,
            _ => panic!("invalid bucket type"),
        }
    }
}
