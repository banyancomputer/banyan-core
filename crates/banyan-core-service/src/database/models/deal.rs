use crate::database::models::deal_state::DealState;

#[derive(sqlx::FromRow)]
pub struct Deal {
    pub id: String,
    pub state: DealState,
    pub size: Option<i64>,
}
