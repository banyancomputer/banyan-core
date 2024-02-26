use crate::database::Database;

#[derive(sqlx::FromRow)]
pub struct Blocks {
    pub id: String,
    pub cid: String,
    pub metadata_id: String,
}

impl Blocks {
    pub async fn get_metadata_id_and_storage_host(
        database: &Database,
        storage_host_id: &str,
    ) -> Result<Vec<Blocks>, sqlx::Error> {
        sqlx::query_as!(
            Blocks,
            "
            SELECT b.id, b.cid, m.id AS metadata_id FROM blocks AS b
                JOIN block_locations AS bl on b.id = bl.block_id
                JOIN metadata AS m ON m.id = bl.metadata_id
            WHERE bl.pruned_at IS NULL AND bl.expired_at IS NULL
                AND bl.storage_host_id = $1;
            ",
            storage_host_id,
        )
        .fetch_all(database)
        .await
    }
}
