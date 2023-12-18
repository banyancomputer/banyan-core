use serde::{Deserialize, Serialize};

use crate::database::models::{Deal, DealState};

#[derive(Serialize, Deserialize)]
pub struct ApiDeal {
    pub id: String,
    pub state: DealState,

    pub size: i64,
}

impl From<Deal> for ApiDeal {
    fn from(value: Deal) -> Self {
        Self {
            id: value.id,
            state: value.state,
            size: value.size.unwrap_or(0),
        }
    }
}
