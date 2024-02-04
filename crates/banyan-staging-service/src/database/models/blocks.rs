use sqlx::FromRow;
use time::OffsetDateTime;

use crate::database::Database;

#[derive(FromRow)]
pub struct Blocks {
    pub id: String,
    pub cid: String,
    pub data_length: i64,
    pub created_at: OffsetDateTime,
}

impl Blocks {
    pub async fn blocks_for_upload(
        conn: &Database,
        upload_id: &str,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let blocks: Vec<Blocks> = sqlx::query_as!(
            Self,
            "SELECT b.* FROM uploads AS u
                JOIN uploads_blocks AS ub ON ub.upload_id = u.id
                JOIN main.blocks b on b.id = ub.block_id
            WHERE pruned_at IS NULL AND u.id = $1;",
            upload_id
        )
        .fetch_all(conn)
        .await?;
        Ok(blocks)
    }
}
