use time::OffsetDateTime;

use crate::database::DatabaseConnection;
use crate::pricing::PricingTier;

pub struct NewSubscription<'a> {
    pub price_key: &'a str,
    pub title: &'a str,

    pub allow_overage: bool,
    pub archival_available: bool,
    pub visible: bool,

    pub base_price: Option<i64>,
    pub storage_overage_price: Option<i64>,
    pub bandwidth_overage_price: Option<i64>,

    pub included_archival: i64,
    pub included_bandwidth: i64,
    pub included_storage: i64,

    pub archival_hard_limit: Option<i64>,
    pub bandwidth_hard_limit: Option<i64>,
    pub storage_hard_limit: Option<i64>,
}

impl NewSubscription<'_> {
    pub async fn save(&self, conn: &mut DatabaseConnection) -> Result<String, sqlx::Error> {
        let now = OffsetDateTime::now_utc();

        let new_sub_id: String = sqlx::query_scalar!(
            r#"INSERT INTO subscriptions (price_key, title, allow_overage, archival_available, visible,
                   base_price, storage_overage_price, bandwidth_overage_price, included_archival,
                   included_bandwidth, included_storage, archival_hard_limit, bandwidth_hard_limit,
                   storage_hard_limit, created_at)
                   VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                   RETURNING id;"#,
            self.price_key,
            self.title,
            self.allow_overage,
            self.archival_available,
            self.visible,
            self.base_price,
            self.storage_overage_price,
            self.bandwidth_overage_price,
            self.included_archival,
            self.included_bandwidth,
            self.included_storage,
            self.archival_hard_limit,
            self.bandwidth_hard_limit,
            self.storage_hard_limit,
            now,
        )
        .fetch_one(&mut *conn)
        .await?;

        // Hide any other versions of the same price key
        sqlx::query_scalar!(
            "UPDATE subscriptions SET visible = false WHERE price_key = ? AND id != ?;",
            self.price_key,
            new_sub_id,
        )
        .execute(&mut *conn)
        .await?;

        // We can delete older subscriptions that never received a stripe product ID (which means
        // no one subscribed to that plan)
        sqlx::query!(
            "DELETE FROM subscriptions WHERE price_key = ? AND id != ? AND stripe_product_id IS NULL;",
            self.price_key,
            new_sub_id,
        )
        .execute(&mut *conn)
        .await?;

        Ok(new_sub_id)
    }
}

impl<'a> From<&'a PricingTier> for NewSubscription<'a> {
    fn from(pricing_tier: &'a PricingTier) -> Self {
        Self {
            price_key: &pricing_tier.price_key,
            title: &pricing_tier.title,

            allow_overage: pricing_tier.allow_overage,
            archival_available: pricing_tier.archival_available,
            visible: pricing_tier.visible,

            base_price: pricing_tier.price.as_ref().map(|p| p.base),
            storage_overage_price: pricing_tier.price.as_ref().map(|p| p.storage_overage),
            bandwidth_overage_price: pricing_tier.price.as_ref().map(|p| p.bandwidth_overage),

            included_archival: pricing_tier.included_allowances.archival,
            included_bandwidth: pricing_tier.included_allowances.bandwidth,
            included_storage: pricing_tier.included_allowances.storage,

            archival_hard_limit: pricing_tier.hard_limits.archival,
            bandwidth_hard_limit: pricing_tier.hard_limits.bandwidth,
            storage_hard_limit: pricing_tier.hard_limits.storage,
        }
    }
}

#[derive(sqlx::FromRow)]
pub struct Subscription {
    id: String,

    price_key: String,
    title: String,

    stripe_product_id: Option<String>,

    allow_overage: bool,
    archival_available: bool,
    visible: bool,

    base_price: Option<i64>,
    storage_overage_price: Option<i64>,
    bandwidth_overage_price: Option<i64>,

    included_archival: i64,
    included_bandwidth: i64,
    included_storage: i64,

    archival_hard_limit: Option<i64>,
    bandwidth_hard_limit: Option<i64>,
    storage_hard_limit: Option<i64>,

    created_at: OffsetDateTime,
}

impl Subscription {
    pub async fn active_price_key(conn: &mut DatabaseConnection, price_key: &str) -> Result<Option<Subscription>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"SELECT * FROM subscriptions
                   WHERE price_key = $1
                   ORDER BY created_at DESC
                   LIMIT 1;"#,
            price_key,
        )
        .fetch_optional(&mut *conn)
        .await
    }
}

impl std::cmp::PartialEq<NewSubscription<'_>> for Subscription {
    fn eq(&self, other: &NewSubscription) -> bool {
        todo!()
    }
}
