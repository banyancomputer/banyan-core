use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::database::models::DealState;

#[derive(Serialize, Deserialize)]
pub struct ApiDeals {
    pub id: String,
    pub state: DealState,
    pub size: i64,
    pub accepted_by: Option<String>,
    pub accepted_at: Option<OffsetDateTime>,
}
