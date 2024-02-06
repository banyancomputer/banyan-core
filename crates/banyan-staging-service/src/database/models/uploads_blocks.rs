use time::OffsetDateTime;

use crate::database::Database;

#[derive(Debug, sqlx::FromRow)]
pub struct UploadsBlocks {
    pub upload_id: String,
    pub block_id: String,
    pub byte_offset: i64,
    pub associated_at: OffsetDateTime,
    pub pruned_at: Option<OffsetDateTime>,
}

impl UploadsBlocks {
    pub async fn mark_as_pruned(conn: &Database, block_id: &String) -> Result<(), sqlx::Error> {
        let pruned_at = OffsetDateTime::now_utc();
        sqlx::query!(
            "UPDATE uploads_blocks SET pruned_at = $1 WHERE block_id = $2",
            pruned_at,
            block_id
        )
        .execute(conn)
        .await?;
        Ok(())
    }
}
