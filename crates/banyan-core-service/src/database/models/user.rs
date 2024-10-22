use serde::Serialize;
use time::OffsetDateTime;

use super::Subscription;
use crate::database::models::{HotUsage, SnapshotState, SubscriptionStatus, TaxClass};
use crate::database::DatabaseConnection;
use crate::utils::GIBIBYTE;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    pub verified_email: bool,
    pub display_name: String,
    pub locale: Option<String>,
    pub region_preference: Option<String>,
    pub profile_image: Option<String>,
    pub created_at: OffsetDateTime,
    pub accepted_tos_at: Option<OffsetDateTime>,
    pub earned_tokens: i64,

    pub account_tax_class: TaxClass,
    pub stripe_customer_id: Option<String>,

    pub stripe_subscription_id: Option<String>,
    pub subscription_id: String,
    pub subscription_status: SubscriptionStatus,
    pub subscription_valid_until: Option<OffsetDateTime>,
}

impl User {
    pub async fn by_id(conn: &mut DatabaseConnection, id: &str) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"
                SELECT id, email, verified_email, display_name, locale, region_preference,
                       profile_image, created_at, accepted_tos_at, earned_tokens,
                       account_tax_class as 'account_tax_class: TaxClass',
                       stripe_customer_id, stripe_subscription_id, subscription_id as 'subscription_id!',
                       subscription_status as 'subscription_status: SubscriptionStatus',
                       subscription_valid_until
                FROM users
                WHERE id = $1;
            "#,
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
    pub async fn hot_usage(&self, conn: &mut DatabaseConnection) -> Result<HotUsage, sqlx::Error> {
        sqlx::query_as!(
            HotUsage,
            r#"
                SELECT
                   COALESCE(SUM(m.metadata_size), 0) as data_size,
                   COALESCE(SUM(COALESCE(m.data_size, m.expected_data_size)), 0) AS meta_size
                FROM metadata m
                INNER JOIN buckets b ON b.id = m.bucket_id
                WHERE b.user_id = $1
                AND b.deleted_at IS NULL
                AND m.state IN ('current', 'outdated', 'pending');
            "#,
            self.id,
        )
        .fetch_one(&mut *conn)
        .await
    }

    pub async fn included_archival(
        &self,
        conn: &mut DatabaseConnection,
    ) -> Result<i64, sqlx::Error> {
        // Grab the user's included archival tokens based on subscription
        Subscription::by_id(&mut *conn, &self.subscription_id)
            .await
            .map(|subscription| subscription.included_archival * GIBIBYTE)
    }

    pub async fn archival_usage(&self, conn: &mut DatabaseConnection) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar!(
            r#"
                SELECT IFNULL(SUM(s.tokens_used), 0)
                FROM snapshots AS s
                JOIN metadata AS m ON m.id = s.metadata_id
                JOIN buckets AS b ON b.id = m.bucket_id
                JOIN users AS u ON u.id = b.user_id
                WHERE u.id = $1
                AND b.deleted_at IS NULL
                AND m.state IN ('current', 'outdated', 'pending')
                AND s.state != $2;
            "#,
            self.id,
            SnapshotState::Error
        )
        .fetch_one(&mut *conn)
        .await
        .map(|t| t as i64)
    }

    pub async fn remaining_tokens(
        &self,
        conn: &mut DatabaseConnection,
    ) -> Result<i64, sqlx::Error> {
        // Ensure that even if a user has earned more than their current plan, they cannot
        // currently take advantage of it
        let available = std::cmp::min(self.included_archival(conn).await?, self.earned_tokens);
        let usage = self.archival_usage(conn).await?;
        Ok(available - usage)
    }

    pub async fn award_tokens(&mut self, conn: &mut DatabaseConnection) -> Result<(), sqlx::Error> {
        let included = self.included_archival(conn).await?;
        let tokens_earned = std::cmp::min(included - self.earned_tokens, included / 6);

        // No need to query if we've already maxxed out
        if tokens_earned == 0 {
            return Ok(());
        }

        sqlx::query!(
            r#"
                UPDATE users
                SET earned_tokens = earned_tokens + $1
                WHERE id = $2;
            "#,
            tokens_earned,
            self.id
        )
        .execute(&mut *conn)
        .await
        .map(|_| {
            self.earned_tokens += tokens_earned;
        })
    }

    pub async fn find_by_id(
        conn: &mut DatabaseConnection,
        id: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            User,
            r#"
                SELECT id, email, verified_email, display_name, locale, region_preference, profile_image, created_at,
                       accepted_tos_at, earned_tokens, account_tax_class as 'account_tax_class: TaxClass',
                       stripe_customer_id, stripe_subscription_id, subscription_id as 'subscription_id!',
                       subscription_status as 'subscription_status: SubscriptionStatus',
                       subscription_valid_until 
                FROM users
                WHERE id = $1;
            "#,
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
            r#"
                SELECT id, email, verified_email, display_name, locale, region_preference, profile_image, created_at,
                       accepted_tos_at, earned_tokens, account_tax_class as 'account_tax_class: TaxClass',
                       stripe_customer_id, stripe_subscription_id, subscription_id as 'subscription_id!',
                       subscription_status as 'subscription_status: SubscriptionStatus',
                       subscription_valid_until
                FROM users
                WHERE stripe_customer_id = $1;"#,
            stripe_customer_id,
        )
        .fetch_optional(&mut *conn)
        .await
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

    pub async fn add_region_preference(
        &mut self,
        conn: &mut DatabaseConnection,
        region: &str,
    ) -> Result<(), sqlx::Error> {
        let not_like = format!("%{}%", region);
        let region_preference = sqlx::query_scalar!(
            r#"
                UPDATE users 
                SET region_preference = IFNULL(region_preference || ',', '') || $1
                WHERE id = $2 
                AND IFNULL(region_preference, '') NOT LIKE $3 
                RETURNING region_preference;
            "#,
            region,
            self.id,
            not_like,
        )
        .fetch_one(&mut *conn)
        .await?;

        self.region_preference = region_preference;

        Ok(())
    }

    pub async fn remove_region_preference(
        &mut self,
        conn: &mut DatabaseConnection,
        region: &str,
    ) -> Result<(), sqlx::Error> {
        // Retrieve the existing preference, falling back on the object
        let existing_preference: String = sqlx::query_scalar!(
            r#"
                SELECT region_preference 
                FROM users 
                WHERE id = $1
                AND region_preference IS NOT NULL;
            "#,
            self.id,
        )
        .fetch_one(&mut *conn)
        .await?
        .unwrap_or(
            self.region_preference
                .clone()
                .ok_or(sqlx::Error::RowNotFound)?,
        );

        // Create the new string using the existing string
        let new_preference = {
            // Filter out any occurrences of the region to remove
            let new_preference = existing_preference
                .split(',')
                .filter(|existing| existing != &region)
                .collect::<Vec<&str>>()
                .join(",");
            if new_preference.is_empty() {
                None
            } else {
                Some(new_preference)
            }
        };

        // Update
        sqlx::query!(
            r#"
                UPDATE users
                SET region_preference = $1
                WHERE id = $2;
            "#,
            new_preference,
            self.id,
        )
        .execute(&mut *conn)
        .await?;

        self.region_preference = new_preference;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::database::models::User;
    use crate::database::test_helpers::*;

    #[tokio::test]
    async fn region_preference_modification() {
        let db = setup_database().await;
        let mut conn = db.acquire().await.expect("connection");

        let user_id = create_user(&mut conn, "example@email.com", "Greg").await;

        // Grab the freshly made user
        let mut user = User::by_id(&mut conn, &user_id).await.expect("user lookup");
        // The user should initially have no preferences
        assert!(user.region_preference.is_none());

        // Add and assert the presence of North America
        user.add_region_preference(&mut conn, "North America")
            .await
            .expect("add preference");
        // Ensure that duplicates do not get added
        assert!(user
            .add_region_preference(&mut conn, "North America")
            .await
            .is_err());
        assert_eq!(user.region_preference, Some(String::from("North America")));

        // Add and Assert the presence of Antarctica
        user.add_region_preference(&mut conn, "Antarctica")
            .await
            .expect("add preference");
        assert_eq!(
            user.region_preference,
            Some(String::from("North America,Antarctica"))
        );

        // Add and Assert the presence of Europe
        user.add_region_preference(&mut conn, "Europe")
            .await
            .expect("add preference");
        assert_eq!(
            user.region_preference,
            Some(String::from("North America,Antarctica,Europe"))
        );

        // Remove and Assert the presence of Europe
        user.remove_region_preference(&mut conn, "Europe")
            .await
            .expect("remove preference");
        assert_eq!(
            user.region_preference,
            Some(String::from("North America,Antarctica"))
        );

        // Remove and Assert the presence of North America
        user.remove_region_preference(&mut conn, "North America")
            .await
            .expect("remove preference");
        assert_eq!(user.region_preference, Some(String::from("Antarctica")));

        // Remove and Assert the presence of Antarctica
        user.remove_region_preference(&mut conn, "Antarctica")
            .await
            .expect("remove preference");
        assert_eq!(user.region_preference, None);
    }
}
