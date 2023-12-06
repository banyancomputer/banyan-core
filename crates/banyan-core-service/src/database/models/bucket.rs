use crate::database::models::{BucketType, StorageClass};
use crate::database::DatabaseConnection;

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
    /// they're fully committed
    pub async fn change_in_progress(
        _conn: &mut DatabaseConnection,
        _bucket_id: &str,
    ) -> Result<bool, sqlx::Error> {
        todo!()
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
        .fetch_optional(conn)
        .await?;

        Ok(found_bucket.is_some())
    }
}

#[cfg(test)]
mod tests {}
