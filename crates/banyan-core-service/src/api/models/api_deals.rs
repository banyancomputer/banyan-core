use serde::{Deserialize, Serialize};

use crate::database::models::{Deal, DealState};

#[derive(Serialize, Deserialize)]
pub struct ApiDeal {
    pub id: String,
    pub state: DealState,

    pub accepted_by: Option<String>,
    pub accepted_at: Option<i64>,

    pub created_at: i64,
    pub updated_at: i64,
}

impl From<Deal> for ApiDeal {
    fn from(value: Deal) -> Self {
        Self {
            id: value.id,
            state: value.state,

            accepted_by: value.accepted_by,
            accepted_at: value.accepted_at.map(|t| t.unix_timestamp()),

            updated_at: value.updated_at.unix_timestamp(),
            created_at: value.created_at.unix_timestamp(),
        }
    }
}
