use serde::{Deserialize, Serialize};

use crate::database::models::{Subscription, TaxClass};

#[derive(Serialize, Deserialize)]
pub struct ApiSubscription {
    id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    currently_active: Option<bool>,

    service_key: String,
    tax_class: TaxClass,
    title: String,

    features: ApiSubscriptionFeatures,

    #[serde(skip_serializing_if = "ApiSubscriptionPricing::is_none")]
    pricing: ApiSubscriptionPricing,

    #[serde(with = "time::serde::rfc3339")]
    test_time: time::OffsetDateTime,
}

impl ApiSubscription {
    pub fn set_active_if_match(&mut self, active_id: &str) {
        self.currently_active = Some(self.id == active_id);
    }
}

#[derive(Serialize, Deserialize)]
pub struct ApiSubscriptionFeatures {
    archival_available: bool,

    included_hot_replica_count: i64,
    included_hot_storage: i64,
    included_bandwidth: i64,

    #[serde(skip_serializing_if = "Option::is_none")]
    archival_hard_limit: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    hot_storage_hard_limit: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    bandwidth_hard_limit: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct ApiSubscriptionPricing {
    #[serde(skip_serializing_if = "Option::is_none")]
    archival: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    hot_storage: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    plan_base: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    bandwidth: Option<f32>,
}

impl ApiSubscriptionPricing {
    pub fn is_none(&self) -> bool {
        self.archival.is_none()
            && self.hot_storage.is_none()
            && self.plan_base.is_none()
            && self.bandwidth.is_none()
    }
}

impl From<Subscription> for ApiSubscription {
    fn from(value: Subscription) -> Self {
        let features = ApiSubscriptionFeatures {
            archival_available: value.archival_available,

            included_hot_replica_count: value.included_hot_replica_count,
            included_hot_storage: value.included_hot_storage,
            included_bandwidth: value.included_bandwidth,

            archival_hard_limit: value.archival_hard_limit,
            hot_storage_hard_limit: value.hot_storage_hard_limit,
            bandwidth_hard_limit: value.bandwidth_hard_limit,
        };

        let pricing = ApiSubscriptionPricing {
            archival: value.archival_price.map(|p| p.in_usd()),
            hot_storage: value.hot_storage_price.map(|p| p.in_usd()),
            plan_base: value.plan_base_price.map(|p| p.in_usd()),
            bandwidth: value.bandwidth_price.map(|p| p.in_usd()),
        };

        Self {
            id: value.id,

            currently_active: None,
            service_key: value.service_key,
            tax_class: value.tax_class,
            title: value.title,

            features,
            pricing,

            test_time: time::OffsetDateTime::now_utc(),
        }
    }
}
