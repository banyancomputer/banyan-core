use sqlx::FromRow;
use time::OffsetDateTime;

use crate::database::Database;

#[derive(FromRow)]
pub struct UploadBlocks {
    pub upload_id: String,
    pub block_id: String,
    pub car_offset: i64,
    pub associated_at: OffsetDateTime,
    pub pruned_at: Option<OffsetDateTime>,
}

impl UploadBlocks {
    pub async fn delete_blocks(conn: &Database, block_ids: Vec<String>) -> Result<(), sqlx::Error> {
        for block_id in block_ids {
            sqlx::query!("DELETE FROM uploads_blocks WHERE block_id = $1", block_id)
                .execute(conn)
                .await?;
        }
        Ok(())
    }
}
