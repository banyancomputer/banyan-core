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
    /// Upgrades a particular metadata version from pending or uploading to the current version.
    /// This method does not allow downgrading and will make no changes if the provided metadata
    /// doesn't match what we're expecting.
    #[tracing::instrument(skip(conn))]
    pub async fn mark_current(
        conn: &mut DatabaseConnection,
        bucket_id: &str,
        metadata_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE metadata SET state = 'current'
                 WHERE bucket_id = $1
                     AND id = $2
                     AND state IN ('pending', 'uploading');",
            bucket_id,
            metadata_id,
        )
        .execute(&mut *conn)
        .await?;

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
    async fn test_expected_marking_current() {
        let db = setup_database().await;
        let mut conn = db.begin().await.expect("connection");

        let user_id = sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;

        let pending_metadata_id = pending_metadata(&mut conn, &bucket_id, 1).await;

        Metadata::mark_current(&mut conn, &bucket_id, &metadata_id).await.expect("marking current");
        assert_metadata_in_state(&mut conn, &pending_metadata_id, MetadataState::Current);
    }

    // marking current for a pending metadata
    // marking current for an uploading metadata
    // marking current with multiple existing current metadata
    // not marking current for other metadata
    // error on not marking metadata current
}
