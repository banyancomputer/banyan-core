use jwt_simple::prelude::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::database::models::{Deal, DealState};

#[derive(Serialize, Deserialize)]
pub struct ApiDealsAdmin {
    pub id: String,
    pub state: DealState,
    pub size: i64,
    pub accepted_by: Option<String>,
    pub accepted_at: Option<OffsetDateTime>,
}
impl From<Deal> for ApiDealsAdmin {
    fn from(value: Deal) -> Self {
        Self {
            id: value.id,
            state: value.state,
            size: value.size,
            accepted_by: value.accepted_by,
            accepted_at: value.accepted_at,
        }
    }
}
