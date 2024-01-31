use banyan_object_store::{ObjectStore, ObjectStorePath};
use time::OffsetDateTime;

use crate::database::models::Bucket;
use crate::database::DatabaseConnection;

/// This struct encompasses the minimum amount of data required to create a new metadata row,
/// omitting data that is populated by the database such as ID and the various timestamps. It
/// should not be used for querying rows out of the database.
pub struct NewMetadata<'a> {
    pub bucket_id: &'a str,

    pub metadata_cid: &'a str,
    pub root_cid: &'a str,

    pub expected_data_size: i64,
}

impl NewMetadata<'_> {
    /// Persists a new row in the metadata table with the associated data. When successful this
    /// returns the ID of the newly created row. By default this initialized the state of the
    /// metadata row to 'uploading', which is the entry-point into the metadata state machine.
    pub async fn save(&self, conn: &mut DatabaseConnection) -> Result<String, sqlx::Error> {
        // Note: default timestamp values are not sufficient, so force the value to be set to a precise form
        // by including the timestamp in the query.
        let now = OffsetDateTime::now_utc();
        sqlx::query_scalar!(
            r#"INSERT INTO metadata (bucket_id, metadata_cid, root_cid, expected_data_size, state, created_at, updated_at)
                   VALUES ($1, $2, $3, $4, 'uploading', $5, $5)
                   RETURNING id;"#,
            self.bucket_id,
            self.metadata_cid,
            self.root_cid,
            self.expected_data_size,
            now,
        )
        .fetch_one(&mut *conn)
        .await
    }
}

pub struct Metadata;

impl Metadata {
    /// Retrieve's the bucket ID associated with the provided metadata ID.
    pub async fn get_bucket_id(
        conn: &mut DatabaseConnection,
        metadata_id: &str,
    ) -> Result<String, sqlx::Error> {
        sqlx::query_scalar!("SELECT bucket_id FROM metadata WHERE id = $1;", metadata_id)
            .fetch_one(&mut *conn)
            .await
    }

    /// Upgrades a particular metadata version from pending or uploading to the current version.
    /// This method does not allow downgrading and will make no changes if the provided metadata
    /// doesn't match what we're expecting.
    #[tracing::instrument(skip(conn))]
    pub async fn mark_current(
        conn: &mut DatabaseConnection,
        bucket_id: &str,
        metadata_id: &str,
        data_size: Option<i64>,
    ) -> Result<(), sqlx::Error> {
        // Note: default timestamp values are not sufficient, so force the value to be set to a precise form
        // by including the timestamp in the query.
        let now = OffsetDateTime::now_utc();
        let current_result = sqlx::query!(
            "UPDATE metadata SET state = 'current', data_size = $3, updated_at = $4
                 WHERE bucket_id = $1
                     AND id = $2
                     AND state IN ('pending', 'uploading');",
            bucket_id,
            metadata_id,
            data_size,
            now,
        )
        .execute(&mut *conn)
        .await?;

        match current_result.rows_affected() {
            // Either the provided ID didn't exist, or it wasn't in a compatible state. Either way
            // indicate that the expected target couldn't be found.
            0 => return Err(sqlx::Error::RowNotFound),
            // This is the "good case", we want there to be exactly one change
            1 => (),
            // Any other number is also an error, but there shouldn't be any way to get here since
            // the ID is specifically included and that is a unique column. If this ever occurs
            // there is likely dramatic database damage or at the very least we can't rely on
            // uniqueness assumptions and _should_ crash.
            _ => unreachable!("query restricted by unique ID"),
        }

        let result = sqlx::query!(
            r#"UPDATE metadata SET state = 'outdated', updated_at = $3
                   WHERE bucket_id = $1
                       AND id != $2
                       AND state = 'current';"#,
            bucket_id,
            metadata_id,
            now,
        )
        .execute(&mut *conn)
        .await?;

        // Zero and one are both expected numbers (new bucket no old one, and existing bucket being
        // replaced). Greater than that and something has gone wonky, warn about the issue but keep
        // going.
        if result.rows_affected() > 1 {
            tracing::warn!(
                expired_metadata_count = result.rows_affected(),
                "multiple metadata versions expired at once"
            );
        }

        // Once a metadata has been marked as current we need to see if there was any bending
        // blocks that are ready to be expired.
        let expired_block_ids = sqlx::query_scalar!(
            "SELECT block_id FROM pending_expirations WHERE metadata_id = $1;",
            metadata_id,
        )
        .fetch_all(&mut *conn)
        .await?;

        Bucket::expire_blocks(&mut *conn, bucket_id, &expired_block_ids).await?;

        Ok(())
    }

    /// Assesses the metadata associated with a given bucket to delete no longer valuable CAR files
    pub async fn delete_outdated(
        conn: &mut DatabaseConnection,
        bucket_id: &str,
        store: &ObjectStore,
    ) -> Result<(), sqlx::Error> {
        // First, query the database to see which metadata need to be deleted
        // We could do this all at once, but we dont want to modify the db unless we've verifiably
        // removed the CAR file, or we'll never be able to hunt down the stragglers
        let deletion_ids: Vec<String> = sqlx::query_scalar(
            r#"
                SELECT id FROM metadata
                    WHERE state = 'outdated'
                    AND id NOT IN (SELECT metadata_id FROM snapshots)
                    ORDER BY updated_at DESC
                    LIMIT -1 OFFSET 5;
            "#,
        )
        .fetch_all(&mut *conn)
        .await?;

        // Start building the query that will actually end up marking the metadata as deleted as
        // well as change their updated_at timestamps
        let mut deletion_query =
            sqlx::QueryBuilder::new("UPDATE metadata SET state = 'deleted', updated_at = ");
        deletion_query.push_bind(OffsetDateTime::now_utc());
        deletion_query.push(" WHERE id IN (");
        let mut separated = deletion_query.separated(", ");

        // Attempt to delete the CAR files associated with these metadata
        let mut successful_deletion_ids = Vec::new();
        for metadata_id in deletion_ids {
            let car_path = ObjectStorePath::from(format!("{}/{}.car", bucket_id, metadata_id));
            if store.delete(&car_path).await.is_ok() {
                successful_deletion_ids.push(metadata_id.clone());
                println!("pushing {metadata_id}");
                separated.push_bind(metadata_id);
            }
        }
        separated.push_unseparated(");");

        // Actually mark the metdata rows as deleted now
        let deletion_result = deletion_query.build().execute(&mut *conn).await?;
        tracing::info!(
            "{} metadata have been marked as deleted and have had their CAR files removed",
            deletion_result.rows_affected()
        );
        Ok(())
    }

    /// Marks a metadata upload as complete. This is only for the metadata, the actual filesystem
    /// content will still need to be uploaded to a storage provider directly which will check in
    /// before making this new version the current one.
    pub async fn upload_complete(
        conn: &mut DatabaseConnection,
        metadata_id: &str,
        metadata_hash: &str,
        metadata_size: i64,
    ) -> Result<(), sqlx::Error> {
        // Note: default timestamp values are not sufficient, so force the value to be set to a precise form
        // by including the timestamp in the query.
        let now = OffsetDateTime::now_utc();
        sqlx::query!(
            r#"UPDATE metadata
                   SET metadata_hash = $2,
                       metadata_size = $3,
                       state = 'pending',
                       updated_at = $4
                   WHERE id = $1;"#,
            metadata_id,
            metadata_hash,
            metadata_size,
            now
        )
        .execute(&mut *conn)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use banyan_object_store::ObjectStoreConnection;
    use bytes::Bytes;
    use std::path::Path;
    use time::macros::datetime;

    use super::*;
    use crate::database::models::{MetadataState as MS, SnapshotState};
    use crate::database::test_helpers::*;

    #[tokio::test]
    async fn mark_current() -> Result<(), sqlx::Error> {
        let db = setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;

        let failed_id = sample_metadata(&mut conn, &bucket_id, 1, MS::UploadFailed).await;
        let deleted_id = sample_metadata(&mut conn, &bucket_id, 2, MS::Deleted).await;
        let pending_id = sample_metadata(&mut conn, &bucket_id, 3, MS::Pending).await;
        let uploading_id = sample_metadata(&mut conn, &bucket_id, 4, MS::Uploading).await;
        let current_id = sample_metadata(&mut conn, &bucket_id, 5, MS::Current).await;
        let metadata_id = sample_metadata(&mut conn, &bucket_id, 6, MS::Pending).await;

        // Only the most recent of these metadata are to me marked current
        Metadata::mark_current(&mut conn, &bucket_id, &metadata_id, None).await?;
        assert_metadata_in_state(&mut conn, &metadata_id, MS::Current).await;
        // Assert that only the formerly current id was marked as outdated
        assert_metadata_in_state(&mut conn, &failed_id, MS::UploadFailed).await;
        assert_metadata_in_state(&mut conn, &deleted_id, MS::Deleted).await;
        assert_metadata_in_state(&mut conn, &pending_id, MS::Pending).await;
        assert_metadata_in_state(&mut conn, &uploading_id, MS::Uploading).await;
        assert_metadata_in_state(&mut conn, &current_id, MS::Outdated).await;

        // The metadata is already outdated, it shouldn't be capable of becoming the current
        // metadata.
        assert!(
            Metadata::mark_current(&mut conn, &bucket_id, &current_id, None)
                .await
                .is_err()
        );

        Ok(())
    }

    #[tokio::test]
    async fn delete_outdated() -> Result<(), sqlx::Error> {
        let db = setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;

        // First, create a bunch of metadata with CAR files, each replacing the previous
        let store: ObjectStore = ObjectStore::new(&ObjectStoreConnection::Local(
            Path::new("./data/uploads").to_path_buf(),
        ))
        .expect("cant initialize objectstore in testing");

        let sample_car_data = Bytes::from_static(b"hello there real data!");
        let metadata_count = 15;
        let mut ids = Vec::new();
        for i in 0..=metadata_count {
            let metadata_id = sample_metadata(&mut conn, &bucket_id, i + 1, MS::Pending).await;
            let path = ObjectStorePath::from(format!("{}/{}.car", bucket_id, metadata_id));
            assert!(store.put(&path, sample_car_data.clone()).await.is_ok());
            Metadata::mark_current(&mut conn, &bucket_id, &metadata_id, None).await?;
            assert_metadata_in_state(&mut conn, &metadata_id, MS::Current).await;

            ids.push(metadata_id);
        }

        // Create a snapshot on one of the metadata, which will be exempt from deletion
        // even though it is one of the oldest metadata rows
        let snapshot_metadata_id = &ids[2];
        let _ = create_snapshot(&mut conn, &snapshot_metadata_id, SnapshotState::Pending).await;

        // Delete outdated CAR files and mark as deleted
        assert!(Metadata::delete_outdated(&mut conn, &bucket_id, &store)
            .await
            .is_ok());

        // Ensure each metadata has the correct state and car properties
        for (index, id) in ids.iter().enumerate() {
            let expected_state = if index == ids.len() - 1 {
                MS::Current
            } else if index >= ids.len() - 6 || id == snapshot_metadata_id {
                MS::Outdated
            } else {
                MS::Deleted
            };
            assert_metadata_in_state(&mut conn, &id, expected_state.clone()).await;
            assert_eq!(
                expected_state == MS::Deleted,
                store
                    .get(&ObjectStorePath::from(format!("{}/{}.car", bucket_id, id)))
                    .await
                    .is_err()
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn missing_metadata_fails() {
        let db = setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;

        let result = Metadata::mark_current(&mut conn, &bucket_id, "fake-id", None).await;
        assert!(result.is_err());
    }

    // TODO: it would be ideal if we could test whether our structs implements the
    // correct use of timestamps, but that's not tenable since we can't control the
    // the timestamp values that they initialize rows with (those are side effects
    // of methods like NewMetadata::save(), Metadata::mark_current(),
    // Metadata::upload_complete()). These are proximate tests that ensure that, at the
    // very least, the timestamps are being set to the correct values by SQLX, and that
    // everything is okay so long as we manually set the timestamp values and don't rely
    // on the defaults. Even then, we test whether defaults are sufficient for
    // timestamps to be deserialized correctly when reading from the database (in other
    // words though defaults have a different format and no precision, they are still
    // interpretable as OffsetDateTime).

    #[tokio::test]
    async fn metadata_timestamps_have_precision() {
        let db = setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;

        let timestamp = datetime!(2021-01-01 00:00:00.000_000_001 UTC);
        let metadata_id = create_metadata(
            &mut conn,
            &bucket_id,
            "root-cid",
            "metadata-cid",
            MS::Current,
            Some(timestamp),
            None,
        )
        .await;

        let (created_at, updated_at) = metadata_timestamps(&mut conn, &metadata_id).await;

        // Assert that the timestamps have utmost precision of nanoseconds
        assert_eq!(created_at, timestamp);
        assert_eq!(updated_at, timestamp);
        assert_eq!(created_at.nanosecond(), 1);
        assert_eq!(updated_at.nanosecond(), 1);
    }

    #[tokio::test]
    async fn metadata_timestamps_have_no_hour_truncation() {
        let db = setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;

        let timestamp = datetime!(2021-01-01 09:59:59.999_999_999 UTC);
        let metadata_id = create_metadata(
            &mut conn,
            &bucket_id,
            "root-cid",
            "metadata-cid",
            MS::Current,
            Some(timestamp),
            None,
        )
        .await;

        let (created_at, updated_at) = metadata_timestamps(&mut conn, &metadata_id).await;

        // Assert that the timestamps have utmost precision of nanoseconds
        assert_eq!(created_at, timestamp);
        assert_eq!(updated_at, timestamp);
        assert_eq!(created_at.nanosecond(), 999_999_999);

        let (raw_created_at, raw_updated_at) =
            raw_metadata_timestamps(&mut conn, &metadata_id).await;

        // Split the text at 'T' and assert the character following the 'T' is a '0'
        assert_eq!(
            raw_created_at
                .split('T')
                .nth(1)
                .unwrap()
                .chars()
                .next()
                .unwrap(),
            '0'
        );
        assert_eq!(
            raw_updated_at
                .split('T')
                .nth(1)
                .unwrap()
                .chars()
                .next()
                .unwrap(),
            '0'
        );
    }

    #[tokio::test]
    async fn metadata_timestamps_have_subsecond_truncation() {
        let db = setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;

        let timestamp = datetime!(2021-01-01 09:59:59.999 UTC);
        let metadata_id = create_metadata(
            &mut conn,
            &bucket_id,
            "root-cid",
            "metadata-cid",
            MS::Current,
            Some(timestamp),
            None,
        )
        .await;

        let (created_at, updated_at) = metadata_timestamps(&mut conn, &metadata_id).await;

        // Assert that the timestamps have utmost precision of nanoseconds
        assert_eq!(created_at, timestamp);
        assert_eq!(updated_at, timestamp);
        assert_eq!(created_at.nanosecond(), 999_000_000);

        let (raw_created_at, raw_updated_at) =
            raw_metadata_timestamps(&mut conn, &metadata_id).await;

        // Get the Text after '.' and before 'Z' and assert that it is '999'
        assert_eq!(
            raw_created_at
                .split('.')
                .nth(1)
                .unwrap()
                .split('Z')
                .next()
                .unwrap(),
            "999"
        );
        assert_eq!(
            raw_updated_at
                .split('.')
                .nth(1)
                .unwrap()
                .split('Z')
                .next()
                .unwrap(),
            "999"
        );
    }

    #[tokio::test]
    async fn default_sqlx_timestamps_have_no_precision_and_are_interpretable() {
        let db = setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;

        // Create a metadata row with no timestamp values -- this should use the defaults
        let root_cid = "root-cid";
        let metadata_cid = "metadata-cid";
        let state = MS::Current;
        let metadata_id = sqlx::query_scalar!(
            r#"INSERT INTO
                    metadata (bucket_id, root_cid, metadata_cid, expected_data_size, state)
                    VALUES ($1, $2, $3, 0, $4)
                    RETURNING id;"#,
            bucket_id,
            root_cid,
            metadata_cid,
            state
        )
        .fetch_one(&mut *conn)
        .await
        .expect("inserting metadata");

        // See if we can read these as OffsetDateTime
        let (created_at, updated_at) = metadata_timestamps(&mut conn, &metadata_id).await;

        // Assert that the timestamps have no nanosecond precision
        assert_eq!(created_at.nanosecond(), 0);
        assert_eq!(updated_at.nanosecond(), 0);

        let (raw_created_at, raw_updated_at) =
            raw_metadata_timestamps(&mut conn, &metadata_id).await;

        // Do simple formatting checks on the raw timestamps using the length of the string
        assert_eq!(raw_created_at.len(), 19);
        assert_eq!(raw_updated_at.len(), 19);

        // Split the text at ' ' and assert that each part has the correct length
        let mut raw_iterable_created_at = raw_created_at.split(' ');
        let mut raw_iterable_updated_at = raw_updated_at.split(' ');

        assert_eq!(raw_iterable_created_at.next().unwrap().len(), 10);
        assert_eq!(raw_iterable_updated_at.next().unwrap().len(), 10);
        assert_eq!(raw_iterable_created_at.next().unwrap().len(), 8);
        assert_eq!(raw_iterable_updated_at.next().unwrap().len(), 8);
    }

    #[tokio::test]
    async fn datetime_now_sqlx_timestamps_have_no_precision_and_are_interpretable() {
        let db = setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;

        let metadata_id = create_metadata(
            &mut conn,
            &bucket_id,
            "root-cid",
            "metadata-cid",
            MS::Current,
            None,
            None,
        )
        .await;

        // Set the timestamps to sqlite values of datetime('now')
        sqlx::query!(
            "UPDATE metadata SET created_at = DATETIME('now'), updated_at = DATETIME('now') WHERE id = $1;",
            metadata_id
        )
        .execute(&mut *conn)
        .await
        .expect("updating timestamps");

        // See if we can read these as OffsetDateTime
        let (created_at, updated_at) = metadata_timestamps(&mut conn, &metadata_id).await;

        // Assert that the timestamps have no nanosecond precision
        assert_eq!(created_at.nanosecond(), 0);
        assert_eq!(updated_at.nanosecond(), 0);

        let (raw_created_at, raw_updated_at) =
            raw_metadata_timestamps(&mut conn, &metadata_id).await;

        // Do simple formatting checks on the raw timestamps using the length of the string
        assert_eq!(raw_created_at.len(), 19);
        assert_eq!(raw_updated_at.len(), 19);

        // Split the text at ' ' and assert that each part has the correct length
        let mut raw_iterable_created_at = raw_created_at.split(' ');
        let mut raw_iterable_updated_at = raw_updated_at.split(' ');

        assert_eq!(raw_iterable_created_at.next().unwrap().len(), 10);
        assert_eq!(raw_iterable_updated_at.next().unwrap().len(), 10);
        assert_eq!(raw_iterable_created_at.next().unwrap().len(), 8);
        assert_eq!(raw_iterable_updated_at.next().unwrap().len(), 8);
    }

    #[tokio::test]
    async fn sqlx_and_precise_timestamps_are_orderable() {
        let db = setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        // Test case where the sqlite timestamp is before the precise timestamp

        let user_id = sample_user(&mut conn, "user-1@domain.tld").await;

        let bucket_id = sample_bucket(&mut conn, &user_id).await;

        // Create a metadata row with no timestamp values -- this should use the defaults
        let root_cid = "root-cid";
        let metadata_cid = "metadata-cid";
        let state = MS::Current;
        let first_metadata_id = sqlx::query_scalar!(
            r#"INSERT INTO
                    metadata (bucket_id, root_cid, metadata_cid, expected_data_size, state)
                    VALUES ($1, $2, $3, 0, $4)
                    RETURNING id;"#,
            bucket_id,
            root_cid,
            metadata_cid,
            state
        )
        .fetch_one(&mut *conn)
        .await
        .expect("inserting metadata");

        // Create a second precisely timestamped metadata row a little bit later
        let timestamp = OffsetDateTime::now_utc() + time::Duration::hours(1);
        let second_metadata_id = create_metadata(
            &mut conn,
            &bucket_id,
            "root-cid",
            "metadata-cid",
            MS::Current,
            Some(timestamp),
            None,
        )
        .await;

        // Query all metadata ids in order of creation
        let metadata_ids = sqlx::query_scalar!("SELECT id FROM metadata ORDER BY created_at ASC;")
            .fetch_all(&mut *conn)
            .await
            .expect("querying metadata ids");

        // Assert that the first metadata id is the first one created
        assert_eq!(metadata_ids[0], first_metadata_id);
        assert_eq!(metadata_ids[1], second_metadata_id);

        // Query all metadata ids in reverse order of creation
        let metadata_ids = sqlx::query_scalar!("SELECT id FROM metadata ORDER BY created_at DESC;")
            .fetch_all(&mut *conn)
            .await
            .expect("querying metadata ids");

        // Assert that the first metadata id is the first one created
        assert_eq!(metadata_ids[0], second_metadata_id);
        assert_eq!(metadata_ids[1], first_metadata_id);

        // Get the OffsetDateTime values for the first and second metadata ids
        let (first_created_at, first_updated_at) =
            metadata_timestamps(&mut conn, &first_metadata_id).await;
        let (second_created_at, second_updated_at) =
            metadata_timestamps(&mut conn, &second_metadata_id).await;

        // Assert that the first metadata id is the first one created
        assert!(first_created_at < second_created_at);
        assert!(first_updated_at < second_updated_at);

        // Test case where the sqlite timestamp is after the precise timestamp

        let user_id = sample_user(&mut conn, "user-2@domain.tld").await;

        let bucket_id = sample_bucket(&mut conn, &user_id).await;

        // Create a piece of metadata with a precise timestamp far in the past
        let timestamp = datetime!(1970-01-01 00:00:00 UTC);
        let first_metadata_id = create_metadata(
            &mut conn,
            &bucket_id,
            "root-cid",
            "metadata-cid",
            MS::Current,
            Some(timestamp),
            None,
        )
        .await;

        // Create a second metadata row with no timestamp values -- this should use the defaults
        let root_cid = "root-cid";
        let metadata_cid = "metadata-cid";
        let state = MS::Current;
        let second_metadata_id = sqlx::query_scalar!(
            r#"INSERT INTO
                    metadata (bucket_id, root_cid, metadata_cid, expected_data_size, state)
                    VALUES ($1, $2, $3, 0, $4)
                    RETURNING id;"#,
            bucket_id,
            root_cid,
            metadata_cid,
            state
        )
        .fetch_one(&mut *conn)
        .await
        .expect("inserting metadata");

        // Query all metadata ids in order of creation
        let metadata_ids = sqlx::query_scalar!(
            "SELECT id FROM metadata WHERE bucket_id = $1 ORDER BY created_at ASC;",
            bucket_id
        )
        .fetch_all(&mut *conn)
        .await
        .expect("querying metadata ids");

        // Assert that the first metadata id is the first one created
        assert_eq!(metadata_ids[0], first_metadata_id);
        assert_eq!(metadata_ids[1], second_metadata_id);

        // Query all metadata ids in reverse order of creation
        let metadata_ids = sqlx::query_scalar!(
            "SELECT id FROM metadata WHERE bucket_id = $1 ORDER BY created_at DESC;",
            bucket_id
        )
        .fetch_all(&mut *conn)
        .await
        .expect("querying metadata ids");

        // Assert that the first metadata id is the first one created
        assert_eq!(metadata_ids[0], second_metadata_id);
        assert_eq!(metadata_ids[1], first_metadata_id);

        // Get the OffsetDateTime values for the first and second metadata ids
        let (first_created_at, first_updated_at) =
            metadata_timestamps(&mut conn, &first_metadata_id).await;
        let (second_created_at, second_updated_at) =
            metadata_timestamps(&mut conn, &second_metadata_id).await;

        // Assert that the first metadata id is the first one created
        assert!(first_created_at < second_created_at);
        assert!(first_updated_at < second_updated_at);
    }
}
