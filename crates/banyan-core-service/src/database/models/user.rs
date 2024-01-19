use serde::Serialize;
use time::OffsetDateTime;

use crate::database::models::ExplicitBigInt;
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

    pub active_subscription_id: String,
    pub stripe_customer_id: Option<String>,

    pub current_stripe_plan_subscription_id: Option<String>,
    pub next_payment_due: Option<OffsetDateTime>,

    pub pending_subscription_id: Option<String>,
    pub pending_subscription_expiration: Option<OffsetDateTime>,
    pub pending_stripe_plan_subscription_id: Option<String>,
}

impl User {
    pub fn active_subscription_id(&self) -> String {
        if let (Some(si), Some(exp)) = (&self.pending_subscription_id, &self.pending_subscription_expiration) {
            if exp >= &OffsetDateTime::now_utc() {
                return si.to_string();
            }
        }

        self.active_subscription_id.clone()
    }

    pub async fn by_id(
        conn: &mut DatabaseConnection,
        id: &str,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"SELECT id, email, verified_email, display_name, locale, profile_image, created_at,
                   accepted_tos_at, active_subscription_id as 'active_subscription_id!',
                   stripe_customer_id, current_stripe_plan_subscription_id, next_payment_due,
                   pending_subscription_id, pending_subscription_expiration,
                   pending_stripe_plan_subscription_id FROM users
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
                    accepted_tos_at, active_subscription_id as 'active_subscription_id!',
                    stripe_customer_id, current_stripe_plan_subscription_id, next_payment_due,
                    pending_subscription_id, pending_subscription_expiration,
                    pending_stripe_plan_subscription_id FROM users
                 WHERE id = $1;"#,
            id,
        )
        .fetch_optional(&mut *conn)
        .await
    }

    pub fn pending_subscription(&self) -> Option<String> {
        match (&self.pending_subscription_id, &self.pending_subscription_expiration) {
            (Some(psi), Some(pse)) if pse >= &OffsetDateTime::now_utc() => Some(psi.to_string()),
            _ => None
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
        subscription_stripe_id: &str,
    ) -> Result<(), sqlx::Error> {
        let expiration = OffsetDateTime::now_utc() + SUBSCRIPTION_CHANGE_EXPIRATION_WINDOW;

        sqlx::query!(
            "UPDATE users SET
                 pending_subscription_id = $1,
                 pending_subscription_expiration = $2,
                 pending_stripe_plan_subscription_id = $3
               WHERE id = $4;",
            subscription_id,
            expiration,
            subscription_stripe_id,
            self.id,
        )
        .execute(&mut *conn)
        .await?;

        self.pending_subscription_id = Some(subscription_id.to_string());
        self.pending_subscription_expiration = Some(expiration);
        self.pending_stripe_plan_subscription_id = Some(subscription_stripe_id.to_string());

        Ok(())
    }
}
