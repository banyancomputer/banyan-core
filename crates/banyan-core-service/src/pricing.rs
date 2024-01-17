use std::sync::OnceLock;

use serde::Deserialize;

use crate::database::DatabaseConnection;
use crate::database::models::NewSubscription;

const CURRENT_PRICING_DATA: &[u8] = include_bytes!("../dist/pricing.ron");

static CURRENT_PRICE_CONFIG: OnceLock<Vec<PricingTier>> = OnceLock::new();

pub fn current_price_config() -> &'static [PricingTier] {
    CURRENT_PRICE_CONFIG.get_or_init(|| {
        ron::de::from_bytes(CURRENT_PRICING_DATA).unwrap()
    })
}

pub async fn sync_pricing(_conn: &mut DatabaseConnection) -> Result<(), sqlx::Error> {
    for pricing_tier in current_price_config() {
        let _new_sub = NewSubscription::from(pricing_tier);

        todo!()
    }

    Ok(())
}

/// How much of each class of metric is considered "part of" the subscription plan. Values are in
/// GiB. If the plan doesn't allow overages this becomes the user's hard limits as they can't pay
/// for extra capacity.
#[derive(Debug, Deserialize)]
pub struct Allowances {
    pub archival: i64,
    pub bandwidth: i64,
    pub storage: i64,
}

/// The absolute maximum amount of each metric that the plan is allowed to consumed. Values are in
/// GiB.
#[derive(Debug, Deserialize)]
pub struct Limits {
    pub archival: Option<i64>,
    pub bandwidth: Option<i64>,
    pub storage: Option<i64>,
}

/// A single currently active price tier that should exist in the database. These will be
/// automatically applied if the settings don't match what is currently in the database.
#[derive(Debug, Deserialize)]
pub struct PricingTier {
    /// A unique string representing a single subscription family that may change price over time,
    /// but we can't automatically upgrade users to new pricing. Users will be associated with a
    /// specific version of this key upon when they sign up or are migrated explicitly to a
    /// different plan.
    pub price_key: String,

    /// A user visible string that will be displayed to the user
    pub title: String,

    /// Whether the plan allows exceeding the bandwidth allowance or storage allowance by paying
    /// for additional usage. Used primarily to prevent the starter packages from being
    /// unrestricted.
    pub allow_overage: bool,

    /// Whether the plan allows snapshots to be taken.
    pub archival_available: bool,

    /// Whether this pricing tier is visible to users through products and pricing APIs. Old
    /// versions of the pricing tier and custom pricing schemes shouldn't be visible unless a user
    /// is explicitly subscribed to them.
    pub visible: bool,

    /// If there are prices associated with this plan this will hold them.
    pub price: Option<Price>,

    /// The thresholds of different types of metered measurements that are included in the monthly
    /// price. Going over any of these thresholds starts acruing overage charges if overages are
    /// allowed. If overages are not allowed this will also be treated as a hard limit.
    pub included_allowances: Allowances,

    /// The upper limits of storage allowed by a particular plan.
    pub hard_limits: Limits,
}

/// The cost associated with a particular [`PricingTier`] if one is set. All values are in
/// decicents ($1.50 == 1_500)
#[derive(Debug, Deserialize)]
pub struct Price {
    /// The base monthly price associated with a particular pricing tier.
    pub base: i64,

    /// The price of each GiB stored in the network beyond the base bandwidth allowance.
    pub storage_overage: i64,

    /// The price of each GiB transferred from the network beyond the base bandwidth allowance.
    pub bandwidth_overage: i64,
}
