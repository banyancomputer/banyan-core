use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Deserialize, Serialize)]
pub struct ApiUser {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub locale: Option<String>,
    pub profile_image: Option<String>,
    pub accepted_tos_at: Option<i64>,

    pub account_tax_class: String,
    pub subscription_id: String,
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "time::serde::rfc3339::option"
    )]
    pub subscription_valid_until: Option<OffsetDateTime>,
    pub available_tokens: i64,
    pub maximum_tokens: i64,
}
