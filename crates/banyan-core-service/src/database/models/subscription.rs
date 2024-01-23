use time::OffsetDateTime;

use crate::database::models::{PriceUnits, TaxClass};
use crate::database::DatabaseConnection;
use crate::pricing::DEFAULT_SUBSCRIPTION_KEY;

#[derive(Clone)]
pub struct NewSubscription<'a> {
    pub service_key: &'a str,
    pub tax_class: TaxClass,
    pub title: &'a str,
    pub visible: bool,

    pub plan_base_price: Option<PriceUnits>,

    pub archival_available: bool,
    pub archival_price: Option<PriceUnits>,
    pub archival_hard_limit: Option<i64>,

    pub hot_storage_price: Option<PriceUnits>,
    pub hot_storage_hard_limit: Option<i64>,

    pub bandwidth_price: Option<PriceUnits>,
    pub bandwidth_hard_limit: Option<i64>,

    pub included_hot_replica_count: i64,
    pub included_hot_storage: i64,
    pub included_bandwidth: i64,
}

impl NewSubscription<'_> {
    pub async fn immutable_create(
        &self,
        conn: &mut DatabaseConnection,
    ) -> Result<String, sqlx::Error> {
        let current =
            match Subscription::active_service(&mut *conn, self.service_key, self.tax_class).await?
            {
                Some(s) => s,
                None => {
                    // Nothing matched, this is a brand new account subscription type
                    return self.save(&mut *conn).await;
                }
            };

        // Compare our built up state against the current one, we only want to create a new version
        // if we differ in a meaningful way from what is there. If we're the same we can just
        // return the ID of the existing one.
        if self == &current {
            return Ok(current.id);
        }

        let new_id = self.save(&mut *conn).await?;

        // I need to check each of the stripe price IDs and inherit the previous version if that
        // portion of the subscriptions hasn't changed. This will also require updating the freshly
        // created record with these IDs (I don't want to make these attributes part of the
        // NewSubscription struct as they shouldn't ever be set from the outside).
        let plan_stripe_id = inheritable_plan_price(&current, self);
        let bandwidth_stripe_id = inheritable_bandwidth_price(&current, self);
        let hot_storage_stripe_id = inheritable_hot_storage_price(&current, self);

        match (plan_stripe_id, bandwidth_stripe_id, hot_storage_stripe_id) {
            // If nothing is inheritable we don't need to make this query
            (None, None, None) => (),
            (psi, bsi, hssi) => {
                sqlx::query!(
                    r#"UPDATE subscriptions
                           SET plan_price_stripe_id = $1,
                               bandwidth_stripe_price_id = $2,
                               hot_storage_stripe_price_id = $3
                         WHERE id = $4;"#,
                    psi,
                    bsi,
                    hssi,
                    new_id,
                )
                .execute(&mut *conn)
                .await?;
            }
        }

        Ok(new_id)
    }

    pub async fn save(&self, conn: &mut DatabaseConnection) -> Result<String, sqlx::Error> {
        let now = OffsetDateTime::now_utc();
        let new_id: String = sqlx::query_scalar!(
            r#"INSERT INTO subscriptions (service_key, tax_class, title, visible, plan_base_price,
                    archival_available, archival_price, archival_hard_limit, hot_storage_price,
                    hot_storage_hard_limit, bandwidth_price, bandwidth_hard_limit,
                    included_hot_replica_count, included_hot_storage, included_bandwidth,
                    created_at
                 ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                 RETURNING id;"#,
            self.service_key,
            self.tax_class,
            self.title,
            self.visible,
            self.plan_base_price,
            self.archival_available,
            self.archival_price,
            self.archival_hard_limit,
            self.hot_storage_price,
            self.hot_storage_hard_limit,
            self.bandwidth_price,
            self.bandwidth_hard_limit,
            self.included_hot_replica_count,
            self.included_hot_storage,
            self.included_bandwidth,
            now,
        )
        .fetch_one(&mut *conn)
        .await?;

        // Hide any other versions of the same price key
        sqlx::query_scalar!(
            "UPDATE subscriptions SET visible = false WHERE service_key = ? AND tax_class = ? AND id != ?;",
            self.service_key,
            self.tax_class,
            new_id,
        )
        .execute(&mut *conn)
        .await?;

        Ok(new_id)
    }
}

/// This comparison only checks the price related settings of a subscription and does not take into
/// account database generated values or remote values (local database ID, stripe product IDs, and
/// creation timestamp specifically).
impl std::cmp::PartialEq<Subscription> for NewSubscription<'_> {
    fn eq(&self, other: &Subscription) -> bool {
        self.service_key == other.service_key
            && self.tax_class == other.tax_class
            && self.title == other.title
            && self.visible == other.visible
            && self.plan_base_price == other.plan_base_price
            && self.archival_available == other.archival_available
            && self.archival_price == other.archival_price
            && self.archival_hard_limit == other.archival_hard_limit
            && self.hot_storage_price == other.hot_storage_price
            && self.hot_storage_hard_limit == other.hot_storage_hard_limit
            && self.bandwidth_price == other.bandwidth_price
            && self.bandwidth_hard_limit == other.bandwidth_hard_limit
            && self.included_hot_replica_count == other.included_hot_replica_count
            && self.included_hot_storage == other.included_hot_storage
            && self.included_bandwidth == other.included_bandwidth
    }
}

#[derive(sqlx::FromRow)]
pub struct Subscription {
    pub id: String,

    pub service_key: String,
    pub tax_class: TaxClass,
    pub title: String,
    pub visible: bool,

    pub plan_base_price: Option<PriceUnits>,
    pub plan_price_stripe_id: Option<String>,

    pub archival_available: bool,
    pub archival_price: Option<PriceUnits>,
    pub archival_stripe_price_id: Option<String>,
    pub archival_hard_limit: Option<i64>,

    pub hot_storage_price: Option<PriceUnits>,
    pub hot_storage_stripe_price_id: Option<String>,
    pub hot_storage_hard_limit: Option<i64>,

    pub bandwidth_price: Option<PriceUnits>,
    pub bandwidth_stripe_price_id: Option<String>,
    pub bandwidth_hard_limit: Option<i64>,

    pub included_hot_replica_count: i64,
    pub included_hot_storage: i64,
    pub included_bandwidth: i64,

    pub created_at: OffsetDateTime,
}

impl Subscription {
    pub async fn active_service(
        conn: &mut DatabaseConnection,
        service_key: &str,
        tax_class: TaxClass,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"SELECT id, service_key, tax_class as 'tax_class: TaxClass', title, visible,
                   plan_base_price as 'plan_base_price: PriceUnits', plan_price_stripe_id,
                   archival_available, archival_price as 'archival_price: PriceUnits',
                   archival_stripe_price_id, archival_hard_limit,
                   hot_storage_price as 'hot_storage_price: PriceUnits', hot_storage_stripe_price_id,
                   hot_storage_hard_limit, bandwidth_price as 'bandwidth_price: PriceUnits',
                   bandwidth_stripe_price_id, bandwidth_hard_limit, included_hot_replica_count,
                   included_hot_storage, included_bandwidth, created_at FROM subscriptions
                 WHERE service_key = $1 AND tax_class = $2 AND visible = true
                 ORDER BY created_at DESC
                 LIMIT 1;"#,
            service_key,
            tax_class,
        )
        .fetch_optional(&mut *conn)
        .await
    }

    /// Returns all the subscriptions currently marked as visible with an option to also include a
    /// specific subscription even if it wouldn't normally be visible. This is used for including a
    /// user's current subscription even if there is a newer variant of that subscription key.
    pub async fn all_public_or_current(
        conn: &mut DatabaseConnection,
        current_id: Option<&str>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"SELECT id, service_key, tax_class as 'tax_class: TaxClass', title, visible,
                   plan_base_price as 'plan_base_price: PriceUnits', plan_price_stripe_id,
                   archival_available, archival_price as 'archival_price: PriceUnits',
                   archival_stripe_price_id, archival_hard_limit,
                   hot_storage_price as 'hot_storage_price: PriceUnits', hot_storage_stripe_price_id,
                   hot_storage_hard_limit, bandwidth_price as 'bandwidth_price: PriceUnits',
                   bandwidth_stripe_price_id, bandwidth_hard_limit, included_hot_replica_count,
                   included_hot_storage, included_bandwidth, created_at FROM subscriptions
                 WHERE visible = true OR ($1 IS NOT NULL AND id = $1);"#,
            current_id,
        )
        .fetch_all(&mut *conn)
        .await
    }

    /// Returns the current subscription ID that should be associated with users by default. There
    /// are assumptions elsewhere that this subscription does not have a price as that requires
    /// active user selection and a workflow to go along with it.
    ///
    /// This differs from other query logic for this type in that it will always return the most
    /// recent subscription of the default type even if that version hasn't been marked public.
    pub async fn default_subscription_id(
        conn: &mut DatabaseConnection,
    ) -> Result<String, sqlx::Error> {
        sqlx::query_scalar!(
            "SELECT id FROM subscriptions WHERE service_key = $1 ORDER BY created_at DESC LIMIT 1;",
            DEFAULT_SUBSCRIPTION_KEY,
        )
        .fetch_one(&mut *conn)
        .await
    }

    pub async fn find_by_id(
        conn: &mut DatabaseConnection,
        subscription_id: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"SELECT id, service_key, tax_class as 'tax_class: TaxClass', title, visible,
                   plan_base_price as 'plan_base_price: PriceUnits', plan_price_stripe_id,
                   archival_available, archival_price as 'archival_price: PriceUnits',
                   archival_stripe_price_id, archival_hard_limit,
                   hot_storage_price as 'hot_storage_price: PriceUnits', hot_storage_stripe_price_id,
                   hot_storage_hard_limit, bandwidth_price as 'bandwidth_price: PriceUnits',
                   bandwidth_stripe_price_id, bandwidth_hard_limit, included_hot_replica_count,
                   included_hot_storage, included_bandwidth, created_at FROM subscriptions
                 WHERE visible = true AND id = $1;"#,
            subscription_id,
        )
        .fetch_optional(&mut *conn)
        .await
    }

    pub async fn persist_bandwidth_price_stripe_id(
        &mut self,
        conn: &mut DatabaseConnection,
        bandwidth_stripe_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE subscriptions SET bandwidth_stripe_price_id = $1 WHERE id = $2;",
            bandwidth_stripe_id,
            self.id
        )
        .execute(&mut *conn)
        .await?;

        self.bandwidth_stripe_price_id = Some(bandwidth_stripe_id.to_string());

        Ok(())
    }

    pub async fn persist_plan_price_stripe_id(
        &mut self,
        conn: &mut DatabaseConnection,
        plan_price_stripe_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE subscriptions SET plan_price_stripe_id = $1 WHERE id = $2;",
            plan_price_stripe_id,
            self.id
        )
        .execute(&mut *conn)
        .await?;

        self.plan_price_stripe_id = Some(plan_price_stripe_id.to_string());

        Ok(())
    }

    pub async fn persist_storage_price_stripe_id(
        &mut self,
        conn: &mut DatabaseConnection,
        storage_stripe_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE subscriptions SET hot_storage_stripe_price_id = $1 WHERE id = $2;",
            storage_stripe_id,
            self.id
        )
        .execute(&mut *conn)
        .await?;

        self.hot_storage_stripe_price_id = Some(storage_stripe_id.to_string());

        Ok(())
    }
}

fn inheritable_bandwidth_price(
    existing_sub: &Subscription,
    new_sub: &NewSubscription,
) -> Option<String> {
    if should_inherit_bandwidth_price(existing_sub, new_sub) {
        existing_sub.bandwidth_stripe_price_id.clone()
    } else {
        None
    }
}

fn inheritable_plan_price(
    existing_sub: &Subscription,
    new_sub: &NewSubscription,
) -> Option<String> {
    if should_inherit_plan_price(existing_sub, new_sub) {
        existing_sub.plan_price_stripe_id.clone()
    } else {
        None
    }
}

fn inheritable_hot_storage_price(
    existing_sub: &Subscription,
    new_sub: &NewSubscription,
) -> Option<String> {
    if should_inherit_hot_storage_price(existing_sub, new_sub) {
        existing_sub.hot_storage_stripe_price_id.clone()
    } else {
        None
    }
}

fn should_inherit_bandwidth_price(existing_sub: &Subscription, new_sub: &NewSubscription) -> bool {
    existing_sub.bandwidth_stripe_price_id.is_some()
        && existing_sub.tax_class == new_sub.tax_class
        && existing_sub.bandwidth_price == new_sub.bandwidth_price
        && existing_sub.included_bandwidth == new_sub.included_bandwidth
}

fn should_inherit_plan_price(existing_sub: &Subscription, new_sub: &NewSubscription) -> bool {
    existing_sub.plan_price_stripe_id.is_some()
        && existing_sub.tax_class == new_sub.tax_class
        && existing_sub.plan_base_price == new_sub.plan_base_price
}

/// Note: This does not take into account hot replica count as that is effectively rolled into the
/// hot storage amount when recorded as part of the price.
fn should_inherit_hot_storage_price(
    existing_sub: &Subscription,
    new_sub: &NewSubscription,
) -> bool {
    existing_sub.hot_storage_stripe_price_id.is_some()
        && existing_sub.tax_class == new_sub.tax_class
        && existing_sub.hot_storage_price == new_sub.hot_storage_price
        && existing_sub.included_hot_storage == new_sub.included_hot_storage
}
