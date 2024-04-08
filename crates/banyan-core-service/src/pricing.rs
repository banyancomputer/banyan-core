use std::sync::OnceLock;
use std::time::Duration;

use serde::Deserialize;

use crate::database::models::{NewSubscription, PriceUnits, TaxClass};
use crate::database::DatabaseConnection;

const BUILTIN_PRICING_DATA: &[u8] = include_bytes!("../data/pricing.ron");

static BUILTIN_PRICING_CONFIG: OnceLock<Vec<PricingTier>> = OnceLock::new();

pub const DEFAULT_SUBSCRIPTION_KEY: &str = "starter";

/// Need to be able to accurately represent increments of $5.6e-07 in whole decimal numbers for
/// fractional minute billing. We'll use a global scaling constant for our currency representation
/// of 10^8.
pub const PRICE_UNIT_TO_USD_RATE: usize = 100_000_000;

pub const PRICE_UNIT_TO_CENTS_RATE: usize = 1_000_000;

/// Subscriptions are confirmed through webhooks once they're passed to stripe. We immediately
/// attempt to switch the user over to the new subscription, but don't allow them to change their
/// subscription again until we get confirmation from stripe or this window of six hours is
/// exceeded (likely indicating a service issue with Stripe).
pub const SUBSCRIPTION_CHANGE_EXPIRATION_WINDOW: Duration = Duration::from_secs(6 * 60 * 60);

/// Sourced from https://stripe.com/docs/tax/tax-codes, this is the tax identifier for business use
/// infrastructure as a service cloud service.
pub const TAX_CODE_BUSINESS: &str = "txcd_10101000";

/// Sourced from https://stripe.com/docs/tax/tax-codes, this is the tax identifier for personal use
/// infrastructure as a service cloud service.
pub const TAX_CODE_PERSONAL: &str = "txcd_10010001";

pub fn builtin_pricing_config() -> &'static [PricingTier] {
    BUILTIN_PRICING_CONFIG.get_or_init(|| ron::de::from_bytes(BUILTIN_PRICING_DATA).unwrap())
}

pub async fn sync_pricing_config(
    conn: &mut DatabaseConnection,
    price_tiers: &[PricingTier],
) -> Result<(), sqlx::Error> {
    for pricing_tier in price_tiers {
        for subscription in pricing_tier.as_subscriptions() {
            subscription.immutable_create(&mut *conn).await?;
        }
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
    pub storage_replicas: i64,
}

/// The absolute maximum amount of each metric that the plan is allowed to consumed. Values are in
/// GiB.
#[derive(Debug, Deserialize)]
pub struct Limits {
    pub archival: Option<i64>,
    pub bandwidth: Option<i64>,
    pub storage: Option<i64>,
}

/// The cost associated with a particular [`PricingTier`] if one is set. All values are in
/// decicents ($1.50 == 1_500)
#[derive(Debug, Deserialize)]
pub struct Price {
    /// The base monthly price associated with a particular pricing tier.
    pub base: PriceUnits,

    /// The price of each GiB stored in archival / cold storage per month. Billed in 6 month
    /// intervals all at once up front.
    pub archival: PriceUnits,

    /// The price of each GiB stored in the network beyond the base bandwidth allowance.
    pub storage: PriceUnits,

    /// The price of each GiB transferred from the network beyond the base bandwidth allowance.
    pub bandwidth: PriceUnits,
}

/// A single currently active price tier that should exist in the database. These will be
/// automatically applied if the settings don't match what is currently in the database.
#[derive(Debug, Deserialize)]
pub struct PricingTier {
    /// A unique string representing a single subscription family that may change price over time.
    /// We can't automatically upgrade users to new pricing. Users will be associated with a
    /// specific version of this key upon when they sign up or are migrated explicitly to a
    /// different plan.
    pub service_key: String,

    /// A user visible string that will be displayed to the user
    pub title: String,

    /// Whether the plan allows exceeding the bandwidth allowance or storage allowance by paying
    /// for additional usage. Used primarily to prevent the starter packages from being
    /// unrestricted.
    pub allow_overages: bool,

    /// Whether the plan allows snapshots to be taken.
    pub archival_available: bool,

    /// If there are prices associated with this plan this will hold them.
    pub price: Option<Price>,

    /// The thresholds of different types of metered measurements that are included in the monthly
    /// price. Going over any of these thresholds starts acruing overage charges if overages are
    /// allowed. If overages are not allowed this will also be treated as a hard limit.
    pub included_allowances: Allowances,

    /// The upper limits of storage allowed by a particular plan.
    pub hard_limits: Limits,
}

impl<'a> PricingTier {
    pub fn as_subscriptions(&'a self) -> Vec<NewSubscription<'a>> {
        let mut subscription = NewSubscription {
            service_key: &self.service_key,
            tax_class: TaxClass::NotApplicable,
            title: &self.title,
            visible: true,

            plan_base_price: self.price.as_ref().map(|p| p.base),

            archival_available: self.archival_available,
            archival_price: self.price.as_ref().map(|p| p.archival),
            archival_hard_limit: self.hard_limits.archival,

            hot_storage_price: self.price.as_ref().map(|p| p.storage),
            hot_storage_hard_limit: self.hard_limits.storage,

            bandwidth_price: self.price.as_ref().map(|p| p.bandwidth),
            bandwidth_hard_limit: self.hard_limits.bandwidth,

            included_hot_replica_count: self.included_allowances.storage_replicas,

            // Internally we account for each replica as storage on its own. To include some number
            // of replicas with a certain amount of storage available we need to multiply them
            // together to find out how much we're actually including when accounting for this.
            included_hot_storage: self.included_allowances.storage_replicas
                * self.included_allowances.storage,
            included_bandwidth: self.included_allowances.bandwidth,
            included_archival: self.included_allowances.archival,
        };

        // A pricing tier without any price doesn't need tax related information and there will
        // only be one. Return a Vec with just this one base sub.
        if self.price.is_none() {
            return vec![subscription];
        }

        // For our subs with prices, we need a personal and a business subscription to
        // differentiate between the two for tax handling.
        if self.price.is_some() {
            subscription.tax_class = TaxClass::Personal;
            let mut business_subscription = subscription.clone();
            business_subscription.tax_class = TaxClass::Business;

            vec![subscription, business_subscription]
        } else {
            vec![subscription]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_key_present_in_pricing_tiers() {
        let builtin_pricing_config = builtin_pricing_config();
        let default_subscription = builtin_pricing_config
            .iter()
            .find(|dc| dc.service_key == DEFAULT_SUBSCRIPTION_KEY);
        assert!(
            default_subscription.is_some(),
            "default subscription to be present"
        );
    }

    /// Users MUST actively choose to be subject to a real pricing plan. Our user creation process
    /// makes the assumption that this is the case and blind associates them with the default
    /// subscription. This test ensures that if those assumptions change, we go and update the
    /// appropriate logic in the app that relies on this assumption.
    #[tokio::test]
    async fn test_default_free_assumption() {
        let builtin_pricing_config = builtin_pricing_config();
        let default_subscription = builtin_pricing_config
            .iter()
            .find(|dc| dc.service_key == DEFAULT_SUBSCRIPTION_KEY)
            .expect("to be present");

        assert!(
            default_subscription.price.is_none(),
            "default subscription to be free"
        );
    }
}
