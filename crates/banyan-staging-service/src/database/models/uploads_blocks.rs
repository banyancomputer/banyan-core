use time::OffsetDateTime;

use crate::database::Database;

#[derive(Debug, sqlx::FromRow)]
pub struct UploadsBlocks {
    pub upload_id: String,
    pub block_cid: String,
    pub byte_offset: i64,
    pub associated_at: OffsetDateTime,
    pub pruned_at: Option<OffsetDateTime>,
}

impl UploadsBlocks {
    pub async fn mark_as_pruned(conn: &Database, block_cid: &String) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE uploads_blocks SET pruned_at = CURRENT_TIMESTAMP from blocks b
                WHERE b.id = uploads_blocks.block_id
              AND b.cid = $1
              AND uploads_blocks.pruned_at IS NULL;",
            block_cid
        )
        .execute(conn)
        .await?;
        Ok(())
    }
}
