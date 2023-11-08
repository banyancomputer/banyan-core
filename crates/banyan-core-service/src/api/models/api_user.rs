use serde::{Deserialize, Serialize};

use crate::database::models::User;

// Represents a User in the Database
#[derive(Deserialize, Serialize)]
pub struct ApiUser {
    pub id: String,
    pub email: String,
    pub verified_email: bool,
    pub display_name: String,
    pub locale: Option<String>,
    pub profile_image: Option<String>,
    pub accepted_tos_at: Option<i64>,
}

impl From<User> for ApiUser {
    fn from(val: User) -> Self {
        Self {
            id: val.id,
            email: val.email,
            verified_email: val.verified_email,
            display_name: val.display_name,
            locale: val.locale,
            profile_image: val.profile_image,
            accepted_tos_at: val.accepted_tos_at.map(|t| t.unix_timestamp()),
        }
    }
}
