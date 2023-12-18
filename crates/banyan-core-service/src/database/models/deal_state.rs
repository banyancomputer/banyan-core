use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, sqlx::Type, PartialEq)]
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
impl Display for DealState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DealState::Active => "active",
                DealState::Accepted => "accepted",
                DealState::Sealed => "sealed",
                DealState::Finalized => "finalized",
                DealState::Cancelled => "cancelled",
            }
        )
    }
}
