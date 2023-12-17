use time::OffsetDateTime;

use crate::database::models::deal_state::DealState;

#[derive(sqlx::FromRow)]
pub struct Deal {
    pub id: String,
    pub state: DealState,

    pub accepted_by: Option<String>,
    pub accepted_at: Option<OffsetDateTime>,

    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}
