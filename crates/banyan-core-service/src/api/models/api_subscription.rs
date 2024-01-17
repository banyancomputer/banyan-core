use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::database::models::Subscription;

#[derive(Serialize, Deserialize)]
pub struct ApiSubscription {
    pub id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub currently_active: Option<bool>,

    pub key: String,
}
