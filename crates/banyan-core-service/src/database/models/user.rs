use serde::Serialize;
use time::OffsetDateTime;

use crate::database::models::{ExplicitBigInt, SubscriptionStatus};
use crate::database::DatabaseConnection;
use crate::pricing::SUBSCRIPTION_CHANGE_EXPIRATION_WINDOW;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    pub verified_email: bool,
    pub display_name: String,
    pub locale: Option<String>,
    pub profile_image: Option<String>,
    pub created_at: OffsetDateTime,
    pub accepted_tos_at: Option<OffsetDateTime>,

    pub stripe_customer_id: Option<String>,

    pub active_stripe_subscription_id: Option<String>,
    pub active_subscription_id: String,
    pub active_subscription_status: SubscriptionStatus,
    pub active_subscription_valid_until: Option<OffsetDateTime>,

    pub pending_stripe_subscription_id: Option<String>,
    pub pending_subscription_id: Option<String>,
    pub pending_subscription_expiration: Option<OffsetDateTime>,
}

impl User {
    pub async fn by_id(conn: &mut DatabaseConnection, id: &str) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"SELECT id, email, verified_email, display_name, locale, profile_image, created_at,
                   accepted_tos_at, stripe_customer_id, active_stripe_subscription_id,
                   active_subscription_id as 'active_subscription_id!',
                   active_subscription_status as 'active_subscription_status: SubscriptionStatus',
                   active_subscription_valid_until, pending_stripe_subscription_id,
                   pending_subscription_id, pending_subscription_expiration FROM users
                 WHERE id = $1;"#,
            id,
        )
        .fetch_one(&mut *conn)
        .await
    }

    /// Retrieves the amount of storage the user is currently known to be consuming or have
    /// reserved at specific storage providers for pending uploads. There are three relevant fields
    /// that need to be considered for this:
    ///
    /// 1. The size of the metadata we're currently storing for the bucket
    /// 2. The finalized sized of data after an upload has been completed at a storage provider
    /// 3. The size reserved for an upload currently in progress
    ///
    /// This measure needs to be updated once blocks are properly expired as we'll need to do
    /// better accounting on older metadata versions that no longer have all their associated
    /// blocks.
    pub async fn consumed_storage(
        conn: &mut DatabaseConnection,
        user_id: &str,
    ) -> Result<i64, sqlx::Error> {
        let ex_size = sqlx::query_as!(
            ExplicitBigInt,
            r#"SELECT
                   COALESCE(SUM(m.metadata_size), 0) +
                     COALESCE(SUM(COALESCE(m.data_size, m.expected_data_size)), 0) AS big_int
                 FROM metadata m
                 INNER JOIN buckets b ON b.id = m.bucket_id
                 WHERE b.user_id = $1 AND m.state IN ('current', 'outdated', 'pending');"#,
            user_id,
        )
        .fetch_one(&mut *conn)
        .await?;

        Ok(ex_size.big_int)
    }

    pub async fn find_by_id(
        conn: &mut DatabaseConnection,
        id: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"SELECT id, email, verified_email, display_name, locale, profile_image, created_at,
                   accepted_tos_at, stripe_customer_id, active_stripe_subscription_id,
                   active_subscription_id as 'active_subscription_id!',
                   active_subscription_status as 'active_subscription_status: SubscriptionStatus',
                   active_subscription_valid_until, pending_stripe_subscription_id,
                   pending_subscription_id, pending_subscription_expiration FROM users
                 WHERE id = $1;"#,
            id,
        )
        .fetch_optional(&mut *conn)
        .await
    }

    pub async fn find_by_stripe_customer_id(
        conn: &mut DatabaseConnection,
        stripe_customer_id: String,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"SELECT id, email, verified_email, display_name, locale, profile_image, created_at,
                   accepted_tos_at, stripe_customer_id, active_stripe_subscription_id,
                   active_subscription_id as 'active_subscription_id!',
                   active_subscription_status as 'active_subscription_status: SubscriptionStatus',
                   active_subscription_valid_until, pending_stripe_subscription_id,
                   pending_subscription_id, pending_subscription_expiration FROM users
                 WHERE stripe_customer_id = $1;"#,
            stripe_customer_id,
        )
        .fetch_optional(&mut *conn)
        .await
    }

    pub fn pending_subscription(&self) -> Option<String> {
        match (
            &self.pending_subscription_id,
            &self.pending_subscription_expiration,
        ) {
            (Some(psi), Some(pse)) if pse >= &OffsetDateTime::now_utc() => Some(psi.to_string()),
            _ => None,
        }
    }

    pub async fn persist_customer_stripe_id(
        &mut self,
        conn: &mut DatabaseConnection,
        customer_stripe_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE users SET stripe_customer_id = $1 WHERE id = $2;",
            customer_stripe_id,
            self.id
        )
        .execute(&mut *conn)
        .await?;

        self.stripe_customer_id = Some(customer_stripe_id.to_string());

        Ok(())
    }

    pub async fn persist_pending_subscription(
        &mut self,
        conn: &mut DatabaseConnection,
        subscription_id: &str,
        stripe_subscription_id: &str,
    ) -> Result<(), sqlx::Error> {
        let expiration = OffsetDateTime::now_utc() + SUBSCRIPTION_CHANGE_EXPIRATION_WINDOW;

        sqlx::query!(
            "UPDATE users SET
                 pending_subscription_id = $1,
                 pending_subscription_expiration = $2,
                 pending_stripe_subscription_id = $3
               WHERE id = $4;",
            subscription_id,
            expiration,
            stripe_subscription_id,
            self.id,
        )
        .execute(&mut *conn)
        .await?;

        self.pending_stripe_subscription_id = Some(stripe_subscription_id.to_string());
        self.pending_subscription_id = Some(subscription_id.to_string());
        self.pending_subscription_expiration = Some(expiration);

        Ok(())
    }
}
