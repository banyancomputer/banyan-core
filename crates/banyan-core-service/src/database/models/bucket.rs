use std::time::Duration;

use crate::database::models::{BucketType, StorageClass};
use crate::database::DatabaseConnection;

/// Used to prevent writes of new metadata versions when there is a newer metadata currently being
/// written. This protection is needed until we can handle merge conflicts and resolve the rapid
/// data only unbatched changes in the client.
pub const METADATA_WRITE_LOCK_SECS: i32 = 30;

/// Sets a threshold for how many dynamic binds we restrict individual queries to. Sqlite has a
/// hard limit here of 65535 binds, but performance impact hits much lower. This value was chosen
/// somewhat arbitrarily and could likely use tuning in the future.
pub const DATABASE_CHUNK_LIMIT: usize = 1024;

/// Internal representation of a "Drive", the name is a holdover from a previous design iteration
/// that referred to these as Buckets. This type is an organization type collecting the contents
/// and versions of the filesystem changes. Content exists as blocks in the storage providers,
/// while the actual filesystem structure and attributes are recorded inside the opaque encrypted
/// Metadata blobs.
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

        let query_result = builder
            .build()
            .persistent(false)
            .execute(&mut *conn)
            .await?;
        let changed_rows = query_result.rows_affected();

        Ok(changed_rows)
    }

    /// Takes that list of blocks, verifies they're associated with the bucket (part of the query),
    /// and marks them as expired so they no longer count against a user's quota and can be
    /// eventually cleaned up.
    ///
    /// Blocks are individually added and associated as metadata uploads complete. When we receive
    /// a new version of the bucket's metadata we also receive a list of blocks that are no longer
    /// active from the client's perspective. Blocks can be associated to multiple buckets so this
    /// needs to be careful to on expire associations specific to a bucket and not others.
    ///
    /// This expects that the provided bucket ID has already been validated to be owned by a user
    /// with appropriate write access.
    #[tracing::instrument(skip(conn, block_list))]
    pub async fn expire_blocks(
        conn: &mut DatabaseConnection,
        bucket_id: &str,
        block_list: impl IntoIterator<Item = &str>,
    ) -> Result<(), sqlx::Error> {
        let mut block_iter = block_list.into_iter().peekable();

        if block_iter.peek().is_none() {
            return Ok(());
        }

        let mut validated_block_list = Vec::new();
        let mut total_block_count = 0;

        while block_iter.peek().is_some() {
            let mut query_builder = sqlx::QueryBuilder::new(
                r#"SELECT b.id FROM blocks AS b
                    JOIN block_locations AS bl ON bl.block_id = b.id
                    JOIN metadata AS m ON m.id = bl.metadata_id
                    WHERE m.bucket_id = $1
                        AND b.cid IN ("#,
            );

            // Chunking size was chosen a bit arbitrarily, sqlx has a bind limit of 65k so we need to
            // make sure this is always below that. This could be increased but there is also a hit
            // when queries get too large.
            let mut chunk_count = 0;
            while let Some(cid) = block_iter.next() {
                query_builder.push("?");
                query_builder.push_bind(cid);

                total_block_count += 1;
                chunk_count += 1;

                if chunk_count > DATABASE_CHUNK_LIMIT {
                    break;
                }

                if block_iter.peek().is_some() {
                    query_builder.push(",");
                }
            }

            query_builder.push(");");
            let query = query_builder.build().persistent(false);

            let block_ids = query.fetch_all(&mut *conn).await?;
            validated_block_list.extend(block_ids);
        }

        // Some of the blocks either didn't exist, we don't know about them yet, or don't belong to
        // the user. We still want to expire the blocks that are valid so this is going to just be
        // a warning to keep an eye on.
        let validated_count = validated_block_list.len();
        if validated_count != total_block_count {
            tracing::warn!(
                validated_count,
                total_block_count,
                "mismatch in expected expired block count"
            );
        }

        todo!()
    }

    /// When a new metadata is pushed to this service we mark it as pending until we receive
    /// appropriate data also uploaded to our storage hosts. Allows checking whether a new metadata
    /// can be written. This will return false only when there is a pending write that is within
    /// the `METADATA_WRITE_LOCK_SECS` window.
    pub async fn is_change_in_progress(
        conn: &mut DatabaseConnection,
        bucket_id: &str,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query_scalar!(
            r#"SELECT created_at FROM metadata
                WHERE bucket_id = $1 AND state = 'current'
                ORDER BY created_at DESC
                LIMIT 1;"#,
            bucket_id,
        )
        .fetch_optional(&mut *conn)
        .await?;

        let created_history_window = format!("-{METADATA_WRITE_LOCK_SECS} seconds");

        let locked_id = if let Some(current_creation_ts) = result {
            // We want to gracefully handle the case where there the "current" version and a
            // "pending" version both live within the METADATA_WRITE_LOCK_SECS window. If the
            // "current" one is newer, the "pending" one shouldn't block a new write.
            sqlx::query_scalar!(
                r#"SELECT id FROM metadata
                       WHERE bucket_id = $1
                           AND created_at > $2
                           AND created_at > DATETIME('now', $3)
                       ORDER BY created_at DESC
                       LIMIT 1;"#,
                bucket_id,
                current_creation_ts,
                created_history_window,
            )
            .fetch_optional(&mut *conn)
            .await?
        } else {
            sqlx::query_scalar!(
                r#"SELECT id FROM metadata
                       WHERE bucket_id = $1
                           AND created_at > DATETIME('now', $2)
                       ORDER BY created_at DESC
                       LIMIT 1;"#,
                bucket_id,
                created_history_window,
            )
            .fetch_optional(&mut *conn)
            .await?
        };

        Ok(locked_id.is_some())
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
        .fetch_optional(&mut *conn)
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
        .fetch_optional(&mut *conn)
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
        .fetch_optional(&mut *conn)
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
    async fn test_is_change_in_progress_check() {
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
