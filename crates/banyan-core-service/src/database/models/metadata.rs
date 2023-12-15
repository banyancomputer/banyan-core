use crate::database::DatabaseConnection;
use crate::database::models::Bucket;

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
        sqlx::query_scalar!(
            r#"INSERT INTO metadata (bucket_id, metadata_cid, root_cid, expected_data_size, state)
                   VALUES ($1, $2, $3, $4, 'uploading')
                   RETURNING id;"#,
            self.bucket_id,
            self.metadata_cid,
            self.root_cid,
            self.expected_data_size,
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
        let current_result = sqlx::query!(
            "UPDATE metadata SET state = 'current', data_size = $3
                 WHERE bucket_id = $1
                     AND id = $2
                     AND state IN ('pending', 'uploading');",
            bucket_id,
            metadata_id,
            data_size,
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
            r#"UPDATE metadata SET state = 'outdated'
                   WHERE bucket_id = $1
                       AND id != $2
                       AND state = 'current';"#,
            bucket_id,
            metadata_id,
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

        let expired_block_iter = expired_block_ids.iter().map(String::as_str);
        Bucket::expire_blocks(&mut *conn, &bucket_id, expired_block_iter).await?;

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
        sqlx::query!(
            r#"UPDATE metadata
                   SET metadata_hash = $2,
                       metadata_size = $3,
                       state = 'pending'
                   WHERE id = $1;"#,
            metadata_id,
            metadata_hash,
            metadata_size,
        )
        .execute(&mut *conn)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::models::MetadataState;
    use crate::database::test_helpers::*;

    #[tokio::test]
    async fn test_expected_getting_marked_current() {
        let db = setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;

        let pending_metadata_id = pending_metadata(&mut conn, &bucket_id, 1).await;
        Metadata::mark_current(&mut conn, &bucket_id, &pending_metadata_id, None)
            .await
            .expect("marking current");
        assert_metadata_in_state(&mut conn, &pending_metadata_id, MetadataState::Current).await;

        let uploading_metadata_id =
            sample_metadata(&mut conn, &bucket_id, 1, MetadataState::Uploading).await;
        Metadata::mark_current(&mut conn, &bucket_id, &uploading_metadata_id, Some(1000))
            .await
            .expect("marking current");
        assert_metadata_in_state(&mut conn, &uploading_metadata_id, MetadataState::Current).await;
        assert_metadata_in_state(&mut conn, &pending_metadata_id, MetadataState::Outdated).await;

        // The metadata is already outdated, it shouldn't be capable of becoming the current
        // metadata.
        let result =
            Metadata::mark_current(&mut conn, &bucket_id, &pending_metadata_id, None).await;
        assert!(result.is_err());
        assert_metadata_in_state(&mut conn, &pending_metadata_id, MetadataState::Outdated).await;
    }

    #[tokio::test]
    async fn test_missing_metadata_fails() {
        let db = setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;

        let result = Metadata::mark_current(&mut conn, &bucket_id, "fake-id", None).await;
        assert!(result.is_err());
    }
}
