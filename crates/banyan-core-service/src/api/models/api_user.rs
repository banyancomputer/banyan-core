use serde::Serialize;

// Represents a User in the Database
#[derive(sqlx::FromRow, Serialize)]
pub struct ApiUser {
    pub id: String,
    pub email: String,
    pub verified_email: bool,
    pub display_name: String,
    pub locale: Option<String>,
    pub profile_image: Option<String>,
}
