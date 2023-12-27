use std::collections::{HashMap, HashSet};
use std::time::Duration;

use banyan_task::TaskLikeExt;
use sqlx::QueryBuilder;
use time::OffsetDateTime;

use crate::database::models::{BucketType, MinimalBlockLocation, StorageClass};
use crate::database::{DatabaseConnection, BIND_LIMIT};
use crate::tasks::PruneBlocksTask;

/// Used to prevent writes of new metadata versions when there is a newer metadata currently being
/// written. This protection is needed until we can handle merge conflicts and resolve the rapid
/// data only unbatched changes in the client.
pub const METADATA_WRITE_LOCK_DURATION: Duration = Duration::from_secs(30);

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
        fingerprints: &[String],
    ) -> Result<u64, sqlx::Error> {
        let mut changed_rows = 0;

        let mut offset_start = 0;
        while offset_start < fingerprints.len() {
            let offset_end =
                std::cmp::min(fingerprints.len().saturating_sub(offset_start), BIND_LIMIT);
            let chunk = &fingerprints[offset_start..offset_end];

            let mut query =
                QueryBuilder::new("UPDATE bucket_keys SET approved = 1 WHERE bucket_id = ");

            query.push_bind(bucket_id);
            query.push(" AND fingerprint IN (");

            let mut separated_values = query.separated(", ");
            for fingerprint in chunk {
                separated_values.push_bind(fingerprint);
            }

            query.push(");");

            let query_result = query.build().persistent(false).execute(&mut *conn).await?;
            changed_rows += query_result.rows_affected();

            offset_start = offset_end;
        }

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
    /// * This expects that the provided bucket ID has already been validated to be owned by a user
    ///   with appropriate write access.
    /// * This expects that all CIDs in the block_list have already been normalized by
    ///   `crate::utils::normalize_cid`.
    ///
    /// Returns a tuple of the number of rows expired and the number of rows that are ready to be
    /// pruned. If a block has a single owner and it gets expired, that block is a candidate for
    /// pruning. If a block has multiple owners, and this association expires all of them or any
    /// remaining ones, the block will become a candidate for pruning as well. A block is only
    /// _not_ a candidate for pruning if there are multiple owners and at least one of the
    /// associations _is not expired_.
    #[tracing::instrument(skip(conn, block_cid_list))]
    pub async fn expire_blocks(
        conn: &mut DatabaseConnection,
        bucket_id: &str,
        block_cid_list: &[String],
    ) -> Result<(u64, u64), sqlx::Error> {
        let mut expired_associations = Vec::new();

        for chunk in block_cid_list.chunks(BIND_LIMIT) {
            let mut query_builder = sqlx::QueryBuilder::new(
                r#"SELECT bl.metadata_id AS metadata_id, bl.block_id AS block_id, bl.storage_host_id AS storage_host_id FROM block_locations AS bl
                       JOIN blocks AS b ON b.id = bl.block_id
                       JOIN metadata AS m ON m.id = bl.metadata_id
                       WHERE bl.expired_at IS NULL
                           AND m.bucket_id = "#,
            );

            query_builder.push_bind(bucket_id);
            query_builder.push(" AND b.cid IN (");

            let mut separated_values = query_builder.separated(", ");
            for block_cid in chunk {
                separated_values.push_bind(block_cid);
            }

            query_builder.push(");");
            let query = query_builder
                .build_query_as::<MinimalBlockLocation>()
                .persistent(false);

            let block_locations = query.fetch_all(&mut *conn).await?;
            expired_associations.extend(block_locations);
        }

        // A short circuit so we know there will be at least one location getting marked for
        // expiration in the following query
        if expired_associations.is_empty() {
            return Ok((0, 0));
        }

        let mut total_rows_expired = 0;
        let mut prune_lists: HashMap<String, HashSet<String>> = HashMap::new();

        // Note: while we're using the same chunk limit, we have 3x the number of binds in this
        // query. This is safe since this limit is far below the threshold that will cause issues.
        for expired_chunk in expired_associations.chunks(BIND_LIMIT / 3) {
            let mut expire_builder = sqlx::QueryBuilder::new(
                r#"UPDATE block_locations SET expired_at = CURRENT_TIMESTAMP
                       WHERE (block_id, metadata_id, storage_host_id) IN "#,
            );

            expire_builder.push_tuples(expired_chunk, |mut b, location| {
                b.push_bind(&location.block_id);
                b.push_bind(&location.metadata_id);
                b.push_bind(&location.storage_host_id);
            });

            expire_builder.push(";");

            let expire_query = expire_builder.build().persistent(false);
            let expire_query_result = expire_query.execute(&mut *conn).await?;
            total_rows_expired += expire_query_result.rows_affected();

            // note: this query is currently incorrect, and need to be rewritten. I need to find
            // block, storage host pairs, where all all the associations are marked as expired and
            // are not already pruned...
            let mut prune_candidate_builder = sqlx::QueryBuilder::new(
                r#"SELECT block_id, storage_host_id FROM block_locations
                       WHERE pruned_at IS NULL AND (block_id, storage_host_id) IN "#,
            );

            prune_candidate_builder.push_tuples(expired_chunk, |mut b, location| {
                b.push_bind(&location.block_id);
                b.push_bind(&location.storage_host_id);
            });

            prune_candidate_builder
                .push(" GROUP BY block_id, storage_host_id HAVING COUNT(*) = COUNT(expired_at);");

            let prune_candidate_query = prune_candidate_builder
                .build_query_as::<PruneCandidate>()
                .persistent(false);

            let prune_candidates = prune_candidate_query.fetch_all(&mut *conn).await?;
            for candidate in prune_candidates {
                prune_lists
                    .entry(candidate.storage_host_id)
                    .or_default()
                    .insert(candidate.block_id);
            }
        }

        let mut total_rows_pruned = 0;
        for (storage_host_id, block_set) in prune_lists.into_iter() {
            let block_list = block_set.into_iter().collect::<Vec<_>>();
            total_rows_pruned += block_list.len();

            let queue_result = PruneBlocksTask::new(storage_host_id, block_list)
                .enqueue_with_connection::<banyan_task::SqliteTaskStore>(&mut *conn)
                .await;

            // A future clean up task can always come back through and catch any blocks not missed.
            // We want to know if the queueing fails, but its not critical enough to abort the
            // expiration transaction.
            if let Err(err) = queue_result {
                tracing::warn!("failed to queue prune block task: {err}");
            }
        }

        Ok((total_rows_expired, total_rows_pruned as u64))
    }

    /// When a new metadata is pushed to this service we mark it as pending until we receive
    /// appropriate data also uploaded to our storage hosts. Allows checking whether a new metadata
    /// can be written. This will return false only when there is a pending write that is within
    /// the `METADATA_WRITE_LOCK_DURATION` window.
    pub async fn is_change_in_progress(
        conn: &mut DatabaseConnection,
        bucket_id: &str,
    ) -> Result<bool, sqlx::Error> {
        let current_ts = sqlx::query_scalar!(
            r#"SELECT created_at FROM metadata
                   WHERE bucket_id = $1 AND state = 'current'
                   ORDER BY created_at DESC
                   LIMIT 1;"#,
            bucket_id,
        )
        .fetch_optional(&mut *conn)
        .await?;

        let lock_window = OffsetDateTime::now_utc() - METADATA_WRITE_LOCK_DURATION;
        let lock_threshold = match current_ts {
            // We both have a "current" metadata to reference and its existence is less than our
            // lock window so use it as our threshold instead of the lock window.
            Some(ts) if ts > lock_window => ts,
            _ => lock_window,
        };

        let locked_id = sqlx::query_scalar!(
            r#"SELECT id FROM metadata
                   WHERE bucket_id = $1
                       AND created_at > $2
                       AND state IN ('pending', 'uploading')
                   ORDER BY created_at DESC
                   LIMIT 1;"#,
            bucket_id,
            lock_threshold,
        )
        .fetch_optional(&mut *conn)
        .await?;

        Ok(locked_id.is_some())
    }

    #[tracing::instrument(skip(conn))]
    pub async fn current_version(
        conn: &mut DatabaseConnection,
        bucket_id: &str,
    ) -> Result<Option<String>, sqlx::Error> {
        let current_result = sqlx::query_scalar!(
            r#"SELECT id FROM metadata
                WHERE bucket_id = $1 AND state = 'current'
                ORDER BY created_at DESC
                LIMIT 1;"#,
            bucket_id,
        )
        .fetch_optional(&mut *conn)
        .await?;

        if let Some(current_id) = current_result {
            return Ok(Some(current_id));
        }

        // Temporary fallback to the newest pending state to work around the client bug overwriting
        // metadata
        let pending_result = sqlx::query_scalar!(
            r#"SELECT id FROM metadata
                   WHERE bucket_id = $1 AND state = 'pending'
                   ORDER BY created_at DESC
                   LIMIT 1;"#,
            bucket_id,
        )
        .fetch_optional(&mut *conn)
        .await?;

        if let Some(pending_id) = &pending_result {
            tracing::warn!(pending_id, "fell back on pending metadata");
        }

        Ok(pending_result)
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

#[derive(Debug, sqlx::FromRow)]
struct PruneCandidate {
    storage_host_id: String,
    block_id: String,
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use time::OffsetDateTime;

    use super::*;
    use crate::database::models::MetadataState;
    use crate::database::test_helpers::*;

    async fn is_bucket_key_approved(
        conn: &mut DatabaseConnection,
        bucket_id: &str,
        fingerprint: &str,
    ) -> Option<bool> {
        sqlx::query_scalar!(
            "SELECT approved FROM bucket_keys WHERE bucket_id = $1 AND fingerprint = $2;",
            bucket_id,
            fingerprint,
        )
        .fetch_optional(&mut *conn)
        .await
        .expect("query success")
    }

    fn time_outside_lock_window() -> OffsetDateTime {
        OffsetDateTime::now_utc() - METADATA_WRITE_LOCK_DURATION - Duration::from_secs(5)
    }

    #[tokio::test]
    async fn test_associated_key_empty_approval() {
        let db = setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;

        create_bucket_key(&mut conn, &bucket_id, "<pubkey>", "001122", false).await;

        Bucket::approve_keys_by_fingerprint(&mut conn, &bucket_id, &Vec::new())
            .await
            .expect("appoval success");

        assert!(!is_bucket_key_approved(&mut conn, &bucket_id, "001122")
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_associated_key_single_approval() {
        let db = setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;

        create_bucket_key(&mut conn, &bucket_id, "<pubkey>", "001122", false).await;
        create_bucket_key(&mut conn, &bucket_id, "<pubkey>", "003355", false).await;

        Bucket::approve_keys_by_fingerprint(&mut conn, &bucket_id, &["003355".to_string()])
            .await
            .expect("appoval success");

        assert!(!is_bucket_key_approved(&mut conn, &bucket_id, "001122")
            .await
            .unwrap());
        assert!(is_bucket_key_approved(&mut conn, &bucket_id, "003355")
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_associated_key_multiple_approval() {
        let db = setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;

        create_bucket_key(&mut conn, &bucket_id, "<pubkey>", "001122", false).await;
        create_bucket_key(&mut conn, &bucket_id, "<pubkey>", "003355", false).await;
        create_bucket_key(&mut conn, &bucket_id, "<pubkey>", "abcdef", false).await;

        let approve_keys = vec!["001122".to_string(), "abcdef".to_string()];
        Bucket::approve_keys_by_fingerprint(&mut conn, &bucket_id, &approve_keys)
            .await
            .expect("appoval success");

        assert!(is_bucket_key_approved(&mut conn, &bucket_id, "001122")
            .await
            .unwrap());
        assert!(!is_bucket_key_approved(&mut conn, &bucket_id, "003355")
            .await
            .unwrap());
        assert!(is_bucket_key_approved(&mut conn, &bucket_id, "abcdef")
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_is_no_metadata_not_in_progress() {
        let db = setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;

        // No metadata instances have yet been uploaded, no change should be in progress
        assert!(!Bucket::is_change_in_progress(&mut conn, &bucket_id)
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_is_current_not_in_progress() {
        let db = setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;

        let base_time = OffsetDateTime::now_utc();
        create_metadata(
            &mut conn,
            &bucket_id,
            "meta-cid",
            "root-cid",
            MetadataState::Current,
            Some(base_time),
        )
        .await;

        // All the metadata is current, no change should be in progress
        assert!(!Bucket::is_change_in_progress(&mut conn, &bucket_id)
            .await
            .unwrap());

        let older_time = base_time - Duration::from_secs(20);
        create_metadata(
            &mut conn,
            &bucket_id,
            "old-meta-cid",
            "old-root-cid",
            MetadataState::Current,
            Some(older_time),
        )
        .await;

        // The pending metadata was created at "before" the current metadata so shouldn't cause the
        // bucket to be considered actively being changed
        assert!(!Bucket::is_change_in_progress(&mut conn, &bucket_id)
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_pending_is_in_progress() {
        let db = setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;

        let base_time = OffsetDateTime::now_utc();
        create_metadata(
            &mut conn,
            &bucket_id,
            "meta-cid",
            "root-cid",
            MetadataState::Pending,
            Some(base_time),
        )
        .await;

        // A just created (within our window) pending metadata should keep the bucket locked as its
        // being changed
        assert!(Bucket::is_change_in_progress(&mut conn, &bucket_id)
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_old_pending_not_in_progress() {
        let db = setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;

        let base_time = time_outside_lock_window();
        create_metadata(
            &mut conn,
            &bucket_id,
            "meta-cid",
            "root-cid",
            MetadataState::Pending,
            Some(base_time),
        )
        .await;

        // A just created (within our window) pending metadata should keep the bucket locked as its
        // being changed
        assert!(!Bucket::is_change_in_progress(&mut conn, &bucket_id)
            .await
            .unwrap());

        // There is a different code path when a 'current' metadata exists, we want to make sure
        // this is still _before_ the pending metadata
        let older_time = base_time - Duration::from_secs(10);
        create_metadata(
            &mut conn,
            &bucket_id,
            "og-meta-cid",
            "og-root-cid",
            MetadataState::Current,
            Some(older_time),
        )
        .await;

        assert!(!Bucket::is_change_in_progress(&mut conn, &bucket_id)
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_uploading_is_in_progress() {
        let db = setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;

        let base_time = OffsetDateTime::now_utc();
        create_metadata(
            &mut conn,
            &bucket_id,
            "meta-cid",
            "root-cid",
            MetadataState::Uploading,
            Some(base_time),
        )
        .await;

        // A just created (within our window) uploading metadata should keep the bucket locked as
        // its being changed.
        assert!(Bucket::is_change_in_progress(&mut conn, &bucket_id)
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_old_uploading_not_in_progress() {
        let db = setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;

        let base_time = time_outside_lock_window();
        create_metadata(
            &mut conn,
            &bucket_id,
            "meta-cid",
            "root-cid",
            MetadataState::Uploading,
            Some(base_time),
        )
        .await;

        // A just created (within our window) pending metadata should keep the bucket locked as its
        // being changed
        assert!(!Bucket::is_change_in_progress(&mut conn, &bucket_id)
            .await
            .unwrap());

        // There is a different code path when a 'current' metadata exists, we want to make sure
        // this is still _before_ the uploading metadata
        let older_time = base_time - Duration::from_secs(10);
        create_metadata(
            &mut conn,
            &bucket_id,
            "og-meta-cid",
            "og-root-cid",
            MetadataState::Current,
            Some(older_time),
        )
        .await;

        assert!(!Bucket::is_change_in_progress(&mut conn, &bucket_id)
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_no_current_metadata_retrieval() {
        let db = setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;

        assert!(Bucket::current_version(&mut conn, &bucket_id)
            .await
            .expect("query success")
            .is_none());
    }

    #[tokio::test]
    async fn test_correct_current_metadata_retrieval() {
        let db = setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;

        // An older pending should have no effect on the current metadata
        let oldest_time = OffsetDateTime::now_utc() - Duration::from_secs(300);
        create_metadata(
            &mut conn,
            &bucket_id,
            "old-meta-cid",
            "old-root-cid",
            MetadataState::Pending,
            Some(oldest_time),
        )
        .await;

        let base_time = OffsetDateTime::now_utc() - Duration::from_secs(300);
        let current_metadata_id = create_metadata(
            &mut conn,
            &bucket_id,
            "meta-cid",
            "root-cid",
            MetadataState::Current,
            Some(base_time),
        )
        .await;

        // An newer pending should have no effect on the current metadata
        let newer_time = OffsetDateTime::now_utc();
        create_metadata(
            &mut conn,
            &bucket_id,
            "new-meta-cid",
            "new-root-cid",
            MetadataState::Pending,
            Some(newer_time),
        )
        .await;

        assert_eq!(
            Bucket::current_version(&mut conn, &bucket_id)
                .await
                .expect("query success"),
            Some(current_metadata_id)
        );
    }

    /// This is temporary behavior to restore access to buckets that were affected by the outdated
    /// metadata bug and should be able to removed rather quickly. It is only triggered in the case
    /// that there is no existing current metadata so will do no harm under normal circumstances.
    #[tokio::test]
    async fn test_pending_metadata_fallback_retrieval() {
        let db = setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;

        let base_time = OffsetDateTime::now_utc();
        let pending_id = create_metadata(
            &mut conn,
            &bucket_id,
            "pm-cid",
            "pr-cid",
            MetadataState::Pending,
            Some(base_time),
        )
        .await;

        assert_eq!(
            Bucket::current_version(&mut conn, &bucket_id)
                .await
                .expect("query success"),
            Some(pending_id)
        );

        // Any current metadata should override the pending one, this creates it in the past as
        // that is a slightly more spicy edge case than a brand new current one
        let older_time = base_time - Duration::from_secs(1800);
        let current_id = create_metadata(
            &mut conn,
            &bucket_id,
            "c-meta-cid",
            "c-root-cid",
            MetadataState::Current,
            Some(older_time),
        )
        .await;

        assert_eq!(
            Bucket::current_version(&mut conn, &bucket_id)
                .await
                .expect("query success"),
            Some(current_id)
        );
    }

    #[tokio::test]
    async fn test_owner_id_checking() {
        let db = setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;

        let owned_by_owner = Bucket::is_owned_by_user_id(&mut conn, &bucket_id, &user_id)
            .await
            .expect("query success");
        assert!(owned_by_owner);

        let other_user_id = sample_user(&mut conn, "other_user@not_domain.com").await;

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

    #[tokio::test]
    async fn test_block_expiration() {
        let db = setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;
        let prim_storage_host_id =
            create_storage_host(&mut conn, "Diskz", "https://127.0.0.1:8001/", 1_000_000).await;
        let bak_storage_host_id =
            create_storage_host(&mut conn, "Bax", "https://127.0.0.1:8002/", 3_000_000).await;

        let first_time = OffsetDateTime::now_utc() - Duration::from_secs(45);
        let initial_metadata_id = create_metadata(
            &mut conn,
            &bucket_id,
            "mcid-1",
            "rcid-1",
            MetadataState::Outdated,
            Some(first_time),
        )
        .await;

        let initial_cids: Vec<_> = normalize_cids(generate_cids(data_generator(0..3))).collect();
        let initial_blocks =
            create_blocks(&mut conn, initial_cids.iter().map(String::as_str)).await;

        associate_blocks(
            &mut conn,
            &initial_metadata_id,
            &prim_storage_host_id,
            initial_blocks.iter().map(String::as_str),
        )
        .await;
        associate_blocks(
            &mut conn,
            &initial_metadata_id,
            &bak_storage_host_id,
            initial_blocks.iter().map(String::as_str),
        )
        .await;

        let current_time = OffsetDateTime::now_utc();
        let following_metadata_id = create_metadata(
            &mut conn,
            &bucket_id,
            "mcid-2",
            "rcid-2",
            MetadataState::Current,
            Some(current_time),
        )
        .await;

        let following_cids: Vec<_> = normalize_cids(generate_cids(data_generator(3..6))).collect();
        let following_blocks =
            create_blocks(&mut conn, following_cids.iter().map(String::as_str)).await;

        associate_blocks(
            &mut conn,
            &following_metadata_id,
            &prim_storage_host_id,
            following_blocks.iter().map(String::as_str),
        )
        .await;
        associate_blocks(
            &mut conn,
            &following_metadata_id,
            &bak_storage_host_id,
            following_blocks.iter().map(String::as_str),
        )
        .await;

        // Test that blocks associated to metadata are untouched when no blocks are provided
        let empty_list: Vec<String> = Vec::new();
        Bucket::expire_blocks(&mut conn, &bucket_id, &empty_list)
            .await
            .expect("expire success");

        let expired_block_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM block_locations WHERE expired_at IS NOT NULL;",
        )
        .fetch_one(&mut *conn)
        .await
        .expect("count success");

        assert_eq!(expired_block_count, 0);

        // Ensure unknown blocks are ignored and don't mess with any existing blocks
        let fake_blocks = vec!["definitely-not-an-id".to_string()];
        Bucket::expire_blocks(&mut conn, &bucket_id, &fake_blocks)
            .await
            .expect("expire success");

        let expired_block_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM block_locations WHERE expired_at IS NOT NULL;",
        )
        .fetch_one(&mut *conn)
        .await
        .expect("count success");

        assert_eq!(expired_block_count, 0);

        // Associate one of the blocks with a different bucket so it doesn't get pruned, but only
        // at a single storage host
        let alt_user_id = sample_user(&mut conn, "alt@domain.tld").await;
        let alt_bucket_id = sample_bucket(&mut conn, &alt_user_id).await;
        let alt_metadata_id = create_metadata(
            &mut conn,
            &alt_bucket_id,
            "amcid",
            "arcid",
            MetadataState::Current,
            None,
        )
        .await;

        let alt_block = vec![initial_blocks[0].as_str()];
        associate_blocks(
            &mut conn,
            &alt_metadata_id,
            &prim_storage_host_id,
            alt_block.into_iter(),
        )
        .await;

        // Test that blocks associated with different versions of metadata are marked as expired when
        // their CID is provided
        let expire_blocks = vec![initial_cids[0].clone(), following_cids[0].clone()];
        let (expired, pruned) = Bucket::expire_blocks(&mut conn, &bucket_id, &expire_blocks)
            .await
            .expect("expire success");

        // Make sure 2x blocks are expired at each of 2x storage hosts
        assert_eq!(expired, 4);

        // Make sure 2x blocks are pruned at one storage host, the other should hold on to one of
        // the two blocks due to the association to the `alt_metadata_id`
        assert_eq!(pruned, 3);
    }
}
