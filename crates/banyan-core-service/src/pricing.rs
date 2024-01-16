use std::sync::OnceLock;

use serde::Deserialize;

use crate::database::DatabaseConnection;

const CURRENT_PRICING_DATA: &[u8] = include_bytes!("../dist/pricing.ron");

static CURRENT_PRICE_CONFIG: OnceLock<Vec<PricingTier>> = OnceLock::new();

pub fn current_price_config() -> &'static [PricingTier] {
    CURRENT_PRICE_CONFIG.get_or_init(|| {
        ron::de::from_bytes(CURRENT_PRICING_DATA).unwrap()
    })
}

pub async fn sync_pricing(_conn: &mut DatabaseConnection) -> Result<(), sqlx::Error> {
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct Allowances {
    archival: usize,
    bandwidth: usize,
    storage: usize,
}

#[derive(Debug, Deserialize)]
pub struct Limits {
    archival: Option<usize>,
    bandwidth: Option<usize>,
    storage: Option<usize>,
}

/// A single currently active price tier that should exist in the database. These will be
/// automatically applied if the settings don't match what is currently in the database.
#[derive(Debug, Deserialize)]
pub struct PricingTier {
    /// A unique string representing a single subscription family that may change price over time,
    /// but we can't automatically upgrade users to new pricing. Users will be associated with a
    /// specific version of this key upon when they sign up or are migrated explicitly to a
    /// different plan.
    key: String,

    /// A user visible string that will be displayed to the user
    title: String,

    /// Whether the plan allows exceeding the bandwidth allowance or storage allowance by paying
    /// for additional usage. Used primarily to prevent the starter packages from being
    /// unrestricted.
    allow_overage: bool,

    /// Whether the plan allows snapshots to be taken.
    archival_available: bool,

    /// Whether this pricing tier is visible to users through products and pricing APIs. Old
    /// versions of the pricing tier and custom pricing schemes shouldn't be visible unless a user
    /// is explicitly subscribed to them.
    visible: bool,

    /// If there are prices associated with this plan this will hold them.
    price: Option<Price>,

    /// The thresholds of different types of metered measurements that are included in the monthly
    /// price. Going over any of these thresholds starts acruing overage charges if overages are
    /// allowed. If overages are not allowed this will also be treated as a hard limit.
    included_allowances: Allowances,

    /// The upper limits of storage allowed by a particular plan.
    hard_limits: Limits,
}

/// The cost associated with a particular [`PricingTier`] if one is set. All values are in
/// decicents ($1.50 == 1_500)
#[derive(Debug, Deserialize)]
pub struct Price {
    /// The base monthly price associated with a particular pricing tier.
    base: usize,

    /// The price of each GiB stored in the network beyond the base bandwidth allowance.
    storage_overage: usize,

    /// The price of each GiB transferred from the network beyond the base bandwidth allowance.
    bandwidth_overage: usize,
}
