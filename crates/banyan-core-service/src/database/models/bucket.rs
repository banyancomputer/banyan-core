use std::time::Duration;

use banyan_task::TaskLikeExt;
use sqlx::QueryBuilder;
use time::OffsetDateTime;

use crate::database::models::{BucketType, StorageClass};
use crate::database::DatabaseConnection;
use crate::tasks::{PruneBlock, PruneBlocksTask};

/// Used to prevent writes of new metadata versions when there is a newer metadata currently being
/// written. This protection is needed until we can handle merge conflicts and resolve the rapid
/// data only unbatched changes in the client.
pub const METADATA_WRITE_LOCK_DURATION: Duration = Duration::from_secs(30);

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
        let mut builder =
            QueryBuilder::new("UPDATE bucket_keys SET approved = 1 WHERE bucket_id = ");

        builder.push_bind(bucket_id);
        builder.push(" AND fingerprint IN (");

        let mut key_iterator = fingerprints.into_iter().peekable();
        while let Some(key) = key_iterator.next() {
            builder.push_bind(key);

            if key_iterator.peek().is_some() {
                builder.push(", ");
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
                    WHERE m.bucket_id = "#,
            );

            query_builder.push_bind(bucket_id);
            query_builder.push(" AND b.cid IN (");

            // Chunking size was chosen a bit arbitrarily, sqlx has a bind limit of 65k so we need to
            // make sure this is always below that. This could be increased but there is also a hit
            // when queries get too large.
            let mut chunk_count = 0;
            while let Some(cid) = block_iter.next() {
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use time::OffsetDateTime;

    use super::*;

    use crate::database::test_helpers;
    use crate::database::models::MetadataState;

    async fn create_bucket_key(
        conn: &mut DatabaseConnection,
        bucket_id: &str,
        public_key: &str,
        fingerprint: &str,
        approved: bool,
    ) -> String {
        sqlx::query_scalar!(
            r#"INSERT INTO bucket_keys (bucket_id, pem, fingerprint, approved)
                   VALUES ($1, $2, $3, $4)
                   RETURNING id;"#,
            bucket_id,
            public_key,
            fingerprint,
            approved,
        )
        .fetch_one(&mut *conn)
        .await
        .expect("successful creation")
    }

    async fn create_metadata_with_state(
        conn: &mut DatabaseConnection,
        bucket_id: &str,
        metadata_cid: &str,
        root_cid: &str,
        state: MetadataState,
        created_at: OffsetDateTime,
    ) -> String {
        sqlx::query_scalar!(
            r#"INSERT INTO metadata (bucket_id, metadata_cid, root_cid, expected_data_size, state,
                       created_at, updated_at)
                   VALUES ($1, $2, $3, 0, $4, $5, $5)
                   RETURNING id;"#,
            bucket_id,
            metadata_cid,
            root_cid,
            state,
            created_at,
        )
        .fetch_one(&mut *conn)
        .await
        .expect("save metadata")
    }

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
        let db = test_helpers::setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = test_helpers::sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = test_helpers::sample_bucket(&mut conn, &user_id).await;

        create_bucket_key(&mut conn, &bucket_id, "<pubkey>", "001122", false).await;

        Bucket::approve_keys_by_fingerprint(&mut conn, &bucket_id, [].into_iter())
            .await
            .expect("appoval success");

        assert!(!is_bucket_key_approved(&mut conn, &bucket_id, "001122").await.unwrap());
    }

    #[tokio::test]
    async fn test_associated_key_single_approval() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = test_helpers::sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = test_helpers::sample_bucket(&mut conn, &user_id).await;

        create_bucket_key(&mut conn, &bucket_id, "<pubkey>", "001122", false).await;
        create_bucket_key(&mut conn, &bucket_id, "<pubkey>", "003355", false).await;

        Bucket::approve_keys_by_fingerprint(&mut conn, &bucket_id, ["003355"].into_iter())
            .await
            .expect("appoval success");

        assert!(!is_bucket_key_approved(&mut conn, &bucket_id, "001122").await.unwrap());
        assert!(is_bucket_key_approved(&mut conn, &bucket_id, "003355").await.unwrap());
    }

    #[tokio::test]
    async fn test_associated_key_multiple_approval() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = test_helpers::sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = test_helpers::sample_bucket(&mut conn, &user_id).await;

        create_bucket_key(&mut conn, &bucket_id, "<pubkey>", "001122", false).await;
        create_bucket_key(&mut conn, &bucket_id, "<pubkey>", "003355", false).await;
        create_bucket_key(&mut conn, &bucket_id, "<pubkey>", "abcdef", false).await;

        let approve_keys = ["001122", "abcdef"].into_iter();
        Bucket::approve_keys_by_fingerprint(&mut conn, &bucket_id, approve_keys)
            .await
            .expect("appoval success");

        assert!(is_bucket_key_approved(&mut conn, &bucket_id, "001122").await.unwrap());
        assert!(!is_bucket_key_approved(&mut conn, &bucket_id, "003355").await.unwrap());
        assert!(is_bucket_key_approved(&mut conn, &bucket_id, "abcdef").await.unwrap());
    }

    #[tokio::test]
    async fn test_is_no_metadata_not_in_progress() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = test_helpers::sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = test_helpers::sample_bucket(&mut conn, &user_id).await;

        // No metadata instances have yet been uploaded, no change should be in progress
        assert!(!Bucket::is_change_in_progress(&mut conn, &bucket_id).await.unwrap());
    }

    #[tokio::test]
    async fn test_is_current_not_in_progress() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = test_helpers::sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = test_helpers::sample_bucket(&mut conn, &user_id).await;

        let base_time = OffsetDateTime::now_utc();
        create_metadata_with_state(&mut conn, &bucket_id, "meta-cid", "root-cid", MetadataState::Current, base_time).await;

        // All the metadata is current, no change should be in progress
        assert!(!Bucket::is_change_in_progress(&mut conn, &bucket_id).await.unwrap());

        let older_time = base_time - Duration::from_secs(20);
        create_metadata_with_state(&mut conn, &bucket_id, "old-meta-cid", "old-root-cid", MetadataState::Current, older_time).await;

        // The pending metadata was created at "before" the current metadata so shouldn't cause the
        // bucket to be considered actively being changed
        assert!(!Bucket::is_change_in_progress(&mut conn, &bucket_id).await.unwrap());
    }

    #[tokio::test]
    async fn test_pending_is_in_progress() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = test_helpers::sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = test_helpers::sample_bucket(&mut conn, &user_id).await;

        let base_time = OffsetDateTime::now_utc();
        create_metadata_with_state(&mut conn, &bucket_id, "meta-cid", "root-cid", MetadataState::Pending, base_time).await;

        // A just created (within our window) pending metadata should keep the bucket locked as its
        // being changed
        assert!(Bucket::is_change_in_progress(&mut conn, &bucket_id).await.unwrap());
    }

    #[tokio::test]
    async fn test_old_pending_not_in_progress() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = test_helpers::sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = test_helpers::sample_bucket(&mut conn, &user_id).await;

        let base_time = time_outside_lock_window();
        create_metadata_with_state(&mut conn, &bucket_id, "meta-cid", "root-cid", MetadataState::Pending, base_time).await;

        // A just created (within our window) pending metadata should keep the bucket locked as its
        // being changed
        assert!(!Bucket::is_change_in_progress(&mut conn, &bucket_id).await.unwrap());

        // There is a different code path when a 'current' metadata exists, we want to make sure
        // this is still _before_ the pending metadata
        let older_time = base_time - Duration::from_secs(10);
        create_metadata_with_state(&mut conn, &bucket_id, "og-meta-cid", "og-root-cid", MetadataState::Current, older_time).await;

        assert!(!Bucket::is_change_in_progress(&mut conn, &bucket_id).await.unwrap());
    }

    #[tokio::test]
    async fn test_uploading_is_in_progress() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = test_helpers::sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = test_helpers::sample_bucket(&mut conn, &user_id).await;

        let base_time = OffsetDateTime::now_utc();
        create_metadata_with_state(&mut conn, &bucket_id, "meta-cid", "root-cid", MetadataState::Uploading, base_time).await;

        // A just created (within our window) uploading metadata should keep the bucket locked as
        // its being changed.
        assert!(Bucket::is_change_in_progress(&mut conn, &bucket_id).await.unwrap());
    }

    #[tokio::test]
    async fn test_old_uploading_not_in_progress() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = test_helpers::sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = test_helpers::sample_bucket(&mut conn, &user_id).await;

        let base_time = time_outside_lock_window();
        create_metadata_with_state(&mut conn, &bucket_id, "meta-cid", "root-cid", MetadataState::Uploading, base_time).await;

        // A just created (within our window) pending metadata should keep the bucket locked as its
        // being changed
        assert!(!Bucket::is_change_in_progress(&mut conn, &bucket_id).await.unwrap());

        // There is a different code path when a 'current' metadata exists, we want to make sure
        // this is still _before_ the uploading metadata
        let older_time = base_time - Duration::from_secs(10);
        create_metadata_with_state(&mut conn, &bucket_id, "og-meta-cid", "og-root-cid", MetadataState::Current, base_time).await;

        assert!(!Bucket::is_change_in_progress(&mut conn, &bucket_id).await.unwrap());
    }

    #[tokio::test]
    async fn test_no_current_metadata_retrieval() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = test_helpers::sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = test_helpers::sample_bucket(&mut conn, &user_id).await;

        assert!(Bucket::current_version(&mut conn, &bucket_id).await.expect("query success").is_none());
    }

    #[tokio::test]
    async fn test_correct_current_metadata_retrieval() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = test_helpers::sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = test_helpers::sample_bucket(&mut conn, &user_id).await;

        // An older pending should have no effect on the current metadata
        let oldest_time = OffsetDateTime::now_utc() - Duration::from_secs(300);
        create_metadata_with_state(&mut conn, &bucket_id, "old-meta-cid", "old-root-cid", MetadataState::Pending, oldest_time).await;

        let base_time = OffsetDateTime::now_utc() - Duration::from_secs(300);
        let current_metadata_id = create_metadata_with_state(&mut conn, &bucket_id, "meta-cid", "root-cid", MetadataState::Current, base_time).await;

        // An newer pending should have no effect on the current metadata
        let newer_time = OffsetDateTime::now_utc();
        create_metadata_with_state(&mut conn, &bucket_id, "new-meta-cid", "new-root-cid", MetadataState::Pending, newer_time).await;

        assert_eq!(Bucket::current_version(&mut conn, &bucket_id).await.expect("query success"), Some(current_metadata_id));
    }

    /// This is temporary behavior to restore access to buckets that were affected by the outdated
    /// metadata bug and should be able to removed rather quickly. It is only triggered in the case
    /// that there is no existing current metadata so will do no harm under normal circumstances.
    #[tokio::test]
    async fn test_pending_metadata_fallback_retrieval() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = test_helpers::sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = test_helpers::sample_bucket(&mut conn, &user_id).await;

        let base_time = OffsetDateTime::now_utc();
        let pending_id = create_metadata_with_state(&mut conn, &bucket_id, "p-meta-cid", "p-root-cid", MetadataState::Pending, base_time).await;

        assert_eq!(Bucket::current_version(&mut conn, &bucket_id).await.expect("query success"), Some(pending_id));

        // Any current metadata should override the pending one, this creates it in the past as
        // that is a slightly more spicy edge case than a brand new current one
        let older_time = base_time - Duration::from_secs(1800);
        let current_id = create_metadata_with_state(&mut conn, &bucket_id, "c-meta-cid", "c-root-cid", MetadataState::Current, older_time).await;

        assert_eq!(Bucket::current_version(&mut conn, &bucket_id).await.expect("query success"), Some(current_id));
    }

    #[tokio::test]
    async fn test_owner_id_checking() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.begin().await.expect("connection");

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

    /// Test that blocks associated with older versions of metadata are untouched when no blocks
    /// are provided
    #[tokio::test]
    #[ignore]
    async fn test_expire_blocks_noop() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = test_helpers::sample_user(&mut conn, "user@domain.tld").await;
        let _bucket_id = test_helpers::sample_bucket(&mut conn, &user_id).await;

        todo!()
    }

    /// Test that blocks associated with older versions of metadata are marked as expired when
    /// their CID is provided
    #[tokio::test]
    #[ignore]
    async fn test_expire_blocks_expected() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = test_helpers::sample_user(&mut conn, "user@domain.tld").await;
        let _bucket_id = test_helpers::sample_bucket(&mut conn, &user_id).await;

        todo!()
    }

    /// Test that blocks associated with older versions of ignore unknown blocks, this should
    /// produce a warning but that kind of side-effect isn't covered in this test.
    #[tokio::test]
    #[ignore]
    async fn test_expire_blocks_unknown() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = test_helpers::sample_user(&mut conn, "user@domain.tld").await;
        let _bucket_id = test_helpers::sample_bucket(&mut conn, &user_id).await;

        todo!()
    }

    /// Test that blocks associated with older versions of metadata are marked as expired when a
    /// non-normalized form of their CID is provided
    #[tokio::test]
    #[ignore]
    async fn test_expire_blocks_normalize_cids() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = test_helpers::sample_user(&mut conn, "user@domain.tld").await;
        let _bucket_id = test_helpers::sample_bucket(&mut conn, &user_id).await;

        todo!()
    }

    /// Test that blocks stored at multiple storage hosts are all marked as expired
    #[tokio::test]
    #[ignore]
    async fn test_expire_blocks_multiple_storage_hosts() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = test_helpers::sample_user(&mut conn, "user@domain.tld").await;
        let _bucket_id = test_helpers::sample_bucket(&mut conn, &user_id).await;

        todo!()
    }

    /// Test that duplicate blocks associated with other buckets are not expired
    #[tokio::test]
    #[ignore]
    async fn test_expire_blocks_safe_inter_bucket() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = test_helpers::sample_user(&mut conn, "user@domain.tld").await;
        let _bucket_id = test_helpers::sample_bucket(&mut conn, &user_id).await;

        todo!()
    }

    /// After changes have been made to expired blocks, we need to queue a background task that
    /// will perform all the tasks involved in cleaning them up.
    #[tokio::test]
    #[ignore]
    async fn test_expire_blocks_queues_prune_task() {
        let db = test_helpers::setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = test_helpers::sample_user(&mut conn, "user@domain.tld").await;
        let _bucket_id = test_helpers::sample_bucket(&mut conn, &user_id).await;

        todo!()
    }
}
