use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::database::models::User;

#[derive(Deserialize, Serialize)]
pub struct ApiUser {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub locale: Option<String>,
    pub profile_image: Option<String>,
    pub accepted_tos_at: Option<i64>,

    pub account_tax_class: String,

    // subscription info
    pub subscription_id: String,
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "time::serde::rfc3339::option"
    )]
    pub subscription_valid_until: Option<OffsetDateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archival_hard_limit: Option<i64>,

    // metrics info
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monthly_ingress: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monthly_egress: Option<i64>,
}
impl ApiUser {
    pub fn with_ingress_egress(mut self, ingress: i64, egress: i64) -> Self {
        self.monthly_ingress = Some(ingress);
        self.monthly_egress = Some(egress);
        self
    }
    pub fn with_archival_hard_limit(mut self, limit: Option<i64>) -> Self {
        self.archival_hard_limit = limit;
        self
    }
}
impl From<User> for ApiUser {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            display_name: user.display_name,
            locale: user.locale,
            profile_image: user.profile_image,
            accepted_tos_at: user.accepted_tos_at.map(|t| t.unix_timestamp()),
            subscription_id: user.subscription_id,
            account_tax_class: user.account_tax_class.to_string(),
            subscription_valid_until: user.subscription_valid_until,
            archival_hard_limit: None,
            monthly_ingress: None,
            monthly_egress: None,
        }
    }
}
