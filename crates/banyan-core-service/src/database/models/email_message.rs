use serde::Serialize;

use crate::database::models::EmailMessageState;
use crate::database::types::PrecisionTimestamp;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct EmailMessage {
    pub id: String,
    pub user_id: String,
    pub sent_at: PrecisionTimestamp,
    pub r#type: String,
    pub state: EmailMessageState,
}
