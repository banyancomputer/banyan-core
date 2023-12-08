use crate::database::DatabaseConnection;

pub struct NewMetadata<'a> {
    bucket_id: &'a str,

    metadata_cid: &'a str,
    root_cid: &'a str,

    expected_data_size: i64,
}

impl NewMetadata<'_> {
    pub async fn save(&self, conn: &DatabaseConnection) -> Result<String, sqlx::Error> {
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
