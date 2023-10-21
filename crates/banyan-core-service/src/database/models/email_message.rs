use serde::Serialize;
use time::OffsetDateTime;

use crate::database::models::EmailMessageState;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct EmailMessage {
    pub id: String,
    pub account_id: String,
    pub sent_at: OffsetDateTime,
    pub r#type: String,
    pub state: EmailMessageState,
}
