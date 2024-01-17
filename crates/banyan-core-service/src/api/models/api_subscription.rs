use serde::{Deserialize, Serialize};

use crate::database::models::Subscription;

#[derive(Serialize, Deserialize)]
pub struct ApiSubscription {
    pub id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub currently_active: Option<bool>,

    pub key: String,
    pub title: String,

    pub allow_overages: bool,
    pub archival_available: bool,

    pub base_price: f32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage_overage_price: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub bandwidth_overage_price: Option<f32>,

    #[serde(skip_serializing_if = "is_zero")]
    pub included_archival: usize,

    #[serde(skip_serializing_if = "is_zero")]
    pub included_bandwidth: usize,

    #[serde(skip_serializing_if = "is_zero")]
    pub included_storage: usize,
}

impl From<Subscription> for ApiSubscription {
    fn from(value: Subscription) -> Self {
        Self {
            id: value.id,

            currently_active: None,
            key: value.price_key,
            title: value.title,

            allow_overages: value.allow_overages,
            archival_available: value.archival_available,

            base_price: value.base_price.map(|p| p as f32 / 1_000.).unwrap_or(0.),
            storage_overage_price: value.storage_overage_price.map(|p| p as f32 / 1_000.),
            bandwidth_overage_price: value.bandwidth_overage_price.map(|p| p as f32 / 1_000.),

            included_archival: value.included_archival as usize,
            included_bandwidth: value.included_bandwidth as usize,
            included_storage: value.included_storage as usize,
        }
    }
}

fn is_zero(val: &usize) -> bool {
    *val == 0
}
