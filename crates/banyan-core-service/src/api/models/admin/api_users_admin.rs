use jwt_simple::prelude::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiUsersAdmin {
    pub id: String,
    pub email: String,
    pub verified_email: bool,
    pub display_name: String,
    pub accepted_tos_at: Option<OffsetDateTime>,
}
