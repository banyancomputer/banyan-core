use time::OffsetDateTime;

use crate::database::models::deal_state::DealState;

#[derive(Debug, sqlx::FromRow)]
pub struct Deal {
    pub id: String,
    pub state: DealState,
    pub size: i64,

    pub accepted_by: Option<String>,
    pub accepted_at: Option<OffsetDateTime>,
}
