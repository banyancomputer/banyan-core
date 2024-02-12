use serde::Serialize;
use time::OffsetDateTime;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Notification {
    pub id: String,
    pub user_id: String,
    pub dismissable: bool,
    pub message: String,
    pub message_key: String,
    pub severity: String,
    pub created_at: OffsetDateTime,
}
