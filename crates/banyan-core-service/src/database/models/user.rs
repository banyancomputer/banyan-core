use serde::Serialize;
use time::OffsetDateTime;

use crate::database::models::{ExplicitBigInt, SubscriptionStatus, TaxClass};
use crate::database::DatabaseConnection;

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
                SELECT id, email, verified_email, display_name, locale, region_preference, profile_image, 
                       created_at, accepted_tos_at, account_tax_class as 'account_tax_class: TaxClass',
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
    pub async fn consumed_storage(
        &self,
        conn: &mut DatabaseConnection,
    ) -> Result<i64, sqlx::Error> {
        let ex_size = sqlx::query_as!(
            ExplicitBigInt,
            r#"SELECT
                   COALESCE(SUM(m.metadata_size), 0) +
                     COALESCE(SUM(COALESCE(m.data_size, m.expected_data_size)), 0) AS big_int
                 FROM metadata m
                 INNER JOIN buckets b ON b.id = m.bucket_id
                 WHERE b.user_id = $1 AND m.state IN ('current', 'outdated', 'pending');"#,
            self.id,
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
            r#"
                SELECT id, email, verified_email, display_name, locale, region_preference, profile_image, created_at,
                       accepted_tos_at, account_tax_class as 'account_tax_class: TaxClass',
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
                       accepted_tos_at, account_tax_class as 'account_tax_class: TaxClass',
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
        let region_preference = sqlx::query_scalar!(
            r#"
                UPDATE users 
                SET region_preference = IFNULL(region_preference || ',', '') || $1
                WHERE id = $2 
                RETURNING region_preference;
            "#,
            region,
            self.id,
        )
        .fetch_one(&mut *conn)
        .await?;

        println!("the region pref was updated to be {:?}", region_preference);

        self.region_preference = region_preference;

        Ok(())
    }

    pub async fn remove_region_preference(
        &mut self,
        conn: &mut DatabaseConnection,
        region: &str,
    ) -> Result<(), sqlx::Error> {
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
        .ok_or(sqlx::Error::RowNotFound)?;

        let new_preference = {
            let new_preference = existing_preference
                .split(",")
                .filter(|existing| existing != &region)
                .collect::<Vec<&str>>()
                .join(",");
            if new_preference.is_empty() {
                None
            } else {
                Some(new_preference)
            }
        };

        println!("existing_preference: {}", existing_preference);
        println!("new_preference: {:?}", new_preference);

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
    use crate::database::{models::User, test_helpers::*};

    #[tokio::test]
    async fn region_preference_modification() {
        let db = setup_database().await;
        let mut conn = db.acquire().await.expect("connection");

        let user_id = create_user(&mut *conn, "example@email.com", "Greg").await;

        // Grab the freshly made user
        let mut user = User::by_id(&mut *conn, &user_id)
            .await
            .expect("user lookup");
        // The user should initially have no preferences
        assert!(user.region_preference.is_none());

        // Add and assert the presence of North America
        user.add_region_preference(&mut *conn, "North America")
            .await
            .expect("add preference");
        assert_eq!(user.region_preference, Some(String::from("North America")));

        // Add and Assert the presence of Antarctica
        user.add_region_preference(&mut *conn, "Antarctica")
            .await
            .expect("add preference");
        assert_eq!(
            user.region_preference,
            Some(String::from("North America,Antarctica"))
        );

        // Add and Assert the presence of Europe
        user.add_region_preference(&mut *conn, "Europe")
            .await
            .expect("add preference");
        assert_eq!(
            user.region_preference,
            Some(String::from("North America,Antarctica,Europe"))
        );

        // Remove and Assert the presence of Europe
        user.remove_region_preference(&mut *conn, "Europe")
            .await
            .expect("remove preference");
        assert_eq!(
            user.region_preference,
            Some(String::from("North America,Antarctica"))
        );

        // Remove and Assert the presence of North America
        user.remove_region_preference(&mut *conn, "North America")
            .await
            .expect("remove preference");
        assert_eq!(user.region_preference, Some(String::from("Antarctica")));

        // Remove and Assert the presence of Antarctica
        user.remove_region_preference(&mut *conn, "Antarctica")
            .await
            .expect("remove preference");
        assert_eq!(user.region_preference, None);
    }
}
