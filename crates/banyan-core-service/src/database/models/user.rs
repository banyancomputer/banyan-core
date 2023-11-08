use serde::Serialize;
use time::OffsetDateTime;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    pub verified_email: bool,
    pub display_name: String,
    pub locale: Option<String>,
    pub profile_image: Option<String>,
    pub created_at: OffsetDateTime,
    pub accepted_tos_at: Option<OffsetDateTime>,
}
