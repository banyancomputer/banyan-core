use serde::Serialize;
use time::OffsetDateTime;

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
        sqlx::query_scalar!(
            r#"SELECT
                COALESCE(SUM(m.metadata_size), 0) + COALESCE(SUM(COALESCE(m.data_size, m.expected_data_size)), 0) AS '!size: i64'
            FROM
                metadata m
            INNER JOIN
                buckets b ON b.id = m.bucket_id
            WHERE
                b.user_id = $1 AND m.state IN ('current', 'outdated', pending');"#,
            user_id,
        )
        .fetch_one(&mut *conn)
        .await
    }
}
