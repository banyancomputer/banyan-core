use serde::Serialize;
use time::OffsetDateTime;

use crate::database::models::ExplicitBigInt;
use crate::database::DatabaseConnection;

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
    pub subscription_id: String,
    pub stripe_customer_id: Option<String>,
}

impl User {
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
                    accepted_tos_at, subscription_id as 'subscription_id!', stripe_customer_id
                    FROM users
                 WHERE id = $1;"#,
            id,
        )
        .fetch_optional(&mut *conn)
        .await
    }
}
