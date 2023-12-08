use std::time::Duration;

use crate::database::models::{BucketType, StorageClass};
use crate::database::DatabaseConnection;

/// The maximum amount of time that a pending metadata update will prevent new updates not in a
/// known chain.
const PENDING_STATUS_WRITE_LOCK_DURATION: Duration = Duration::from_secs(15);

#[derive(sqlx::FromRow)]
pub struct Bucket {
    pub id: String,

    pub user_id: String,

    pub name: String,
    pub r#type: BucketType,
    pub storage_class: StorageClass,
}

impl Bucket {
    /// For a particular bucket mark keys with the fingerprints contained within as having been
    /// approved for use with that bucket. We can't verify the key payload correctly contains valid
    /// copies of the inner filesystem key, so there is a little bit of trust here. Key lifecycle
    /// details should be documented elsewhere.
    ///
    /// Hazard: This does not check if the length of the iterator is over the bind limit supported
    /// by sqlx or the database. If the iterator returns > 60k entries these calls will fail with
    /// an obtuse error.
    pub async fn approve_keys_by_fingerprint(
        conn: &mut DatabaseConnection,
        bucket_id: &str,
        fingerprints: impl IntoIterator<Item = &str>,
    ) -> Result<u64, sqlx::Error> {
        let mut builder = sqlx::QueryBuilder::new(
            "UPDATE bucket_keys SET approved = 1 WHERE bucket_id = $1 AND fingerprint IN (",
        );
        builder.push_bind(bucket_id);

        let mut key_iterator = fingerprints.into_iter().peekable();
        while let Some(key) = key_iterator.next() {
            builder.push("?");
            builder.push_bind(key);

            if key_iterator.peek().is_some() {
                builder.push(",");
            }
        }

        builder.push(");");

        let query_result = builder.build().execute(&mut *conn).await?;
        let changed_rows = query_result.rows_affected();

        Ok(changed_rows)
    }

    /// When a new metadata is pushed to this service we mark it as pending until we receive
    /// appropriate data also uploaded to our storage hosts. To prevent overwrites of data before
    /// they're fully committed. This is gated with an arbitrary timeout now as a stop-gap against
    /// aggressive overwriting by our clients.
    pub async fn change_in_progress(
        conn: &mut DatabaseConnection,
        bucket_id: &str,
    ) -> Result<bool, sqlx::Error> {
        todo!()
    }

    #[tracing::instrument(skip(conn))]
    pub async fn current_version(
        conn: &mut DatabaseConnection,
        bucket_id: &str,
    ) -> Result<Option<String>, sqlx::Error> {
        let result = sqlx::query_scalar!(
            r#"SELECT id FROM metadata
                WHERE bucket_id = $1 AND state = 'current'
                ORDER BY created_at DESC
                LIMIT 1;"#,
            bucket_id,
        )
        .fetch_optional(conn)
        .await?;

        if let Some(current_id) = result {
            return Ok(Some(current_id));
        }

        // Temporary fallback to the newest pending state to work around the client bug overwriting
        // metadata

        let result = sqlx::query_scalar!(
            r#"SELECT id FROM metadata
                   WHERE bucket_id = $1 AND state = 'pending'
                   ORDER BY created_at ASC
                   LIMIT 1;"#,
            bucket_id,
        )
        .fetch_optional(conn)
        .await?;

        if let Some(pending_id) = result {
            tracing::warn!(pending_id, "fell back on pending metadata")
        }

        Ok(None)
    }

    /// Checks whether the provided bucket ID is owned by the provided user ID. This will return
    /// false when the user IDs don't match, but also if the bucket doesn't exist (and the user
    /// inherently doesn't the unknown ID).
    pub async fn is_owned_by_user_id(
        conn: &mut DatabaseConnection,
        bucket_id: &str,
        user_id: &str,
    ) -> Result<bool, sqlx::Error> {
        let found_bucket = sqlx::query_scalar!(
            "SELECT id FROM buckets WHERE id = $1 AND user_id = $2;",
            bucket_id,
            user_id,
        )
        .fetch_optional(&mut conn)
        .await?;

        Ok(found_bucket.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::database::test_helpers;

    #[tokio::test]
    async fn test_associated_key_approval() {
        todo!()
    }

    #[tokio::test]
    async fn test_change_in_progress_check() {
        todo!()
    }

    #[tokio::test]
    async fn test_current_metadata_retrieval() {
        todo!();
    }

    #[tokio::test]
    async fn test_pending_fallback_metadata_retrieval() {
        todo!();
    }

    #[tokio::test]
    async fn test_owner_id_checking() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.acquire().await.expect("connection");

        let user_id = test_helpers::sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = test_helpers::sample_bucket(&mut conn, &user_id).await;

        let owned_by_owner = Bucket::is_owned_by_user_id(&mut conn, &bucket_id, &user_id)
            .await
            .expect("query success");
        assert!(owned_by_owner);

        let other_user_id = test_helpers::sample_user(&mut conn, "other_user@not_domain.com").await;

        let owned_by_other = Bucket::is_owned_by_user_id(&mut conn, &bucket_id, &other_user_id)
            .await
            .expect("query success");
        assert!(!owned_by_other);

        let unknown_bucket_owner =
            Bucket::is_owned_by_user_id(&mut conn, "non-existent", &other_user_id)
                .await
                .expect("query success");
        assert!(!unknown_bucket_owner);
    }
}
