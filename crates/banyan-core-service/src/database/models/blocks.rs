use sqlx::sqlite::SqliteQueryResult;

use crate::database::{Database, DatabaseConnection};

#[derive(sqlx::FromRow)]
pub struct Blocks {
    pub id: String,
    pub cid: String,
    pub metadata_id: String,
}

impl Blocks {
    pub async fn insert_block_cid(
        transaction: &mut DatabaseConnection,
        block_cid: &str,
    ) -> Result<SqliteQueryResult, sqlx::Error> {
        sqlx::query!("INSERT OR IGNORE INTO blocks (cid) VALUES ($1);", block_cid)
            .execute(&mut *transaction)
            .await
    }

    pub async fn get_block_id(
        transaction: &mut DatabaseConnection,
        block_cid: &str,
    ) -> Result<String, sqlx::Error> {
        sqlx::query_scalar!("SELECT id FROM blocks WHERE cid = $1", block_cid)
            .fetch_one(&mut *transaction)
            .await
    }
    pub async fn get_blocks_requiring_sync(
        database: &Database,
        storage_host_id: &str,
    ) -> Result<Vec<Blocks>, sqlx::Error> {
        sqlx::query_as!(
            Blocks,
            "SELECT b.id, b.cid, m.id AS metadata_id
            FROM blocks AS b
                JOIN block_locations AS bl on b.id = bl.block_id
                JOIN metadata AS m ON m.id = bl.metadata_id
            WHERE bl.pruned_at IS NULL
                AND bl.expired_at IS NULL
                AND bl.storage_host_id = $1
                AND b.id NOT IN (
                    SELECT bl2.block_id
                    FROM block_locations AS bl2
                    WHERE bl2.storage_host_id != $1);
            ",
            storage_host_id
        )
        .fetch_all(database)
        .await
    }
    pub async fn get_block_ids(
        transaction: &mut DatabaseConnection,
        normalized_cids: &[String],
    ) -> Result<Vec<String>, sqlx::Error> {
        let mut prune_builder = sqlx::QueryBuilder::new("SELECT id FROM blocks WHERE cid IN(");

        let mut block_id_iterator = normalized_cids.iter().peekable();
        while let Some(bid) = block_id_iterator.next() {
            prune_builder.push_bind(bid);

            if block_id_iterator.peek().is_some() {
                prune_builder.push(", ");
            }
        }
        prune_builder.push(");");

        prune_builder
            .build_query_scalar()
            .fetch_all(&mut *transaction)
            .await
    }
}
