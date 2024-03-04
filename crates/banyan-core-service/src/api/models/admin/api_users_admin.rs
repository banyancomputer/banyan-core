use jwt_simple::prelude::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::database::models::User;

#[derive(Serialize, Deserialize)]
pub struct ApiUsersAdmin {
    pub id: String,
    pub email: String,
    pub verified_email: bool,
    pub display_name: String,
    pub accepted_tos_at: Option<OffsetDateTime>,
}
impl From<User> for ApiUsersAdmin {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            email: value.email,
            verified_email: value.verified_email,
            display_name: value.display_name,
            accepted_tos_at: value.accepted_tos_at,
        }
    }
}
