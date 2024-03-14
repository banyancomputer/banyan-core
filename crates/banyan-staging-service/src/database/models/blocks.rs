use sqlx::sqlite::SqliteQueryResult;
use sqlx::FromRow;
use time::OffsetDateTime;

use crate::database::{Database, DatabaseConnection};

#[derive(FromRow)]
pub struct Blocks {
    pub id: String,
    pub cid: String,
    pub data_length: i64,
    pub created_at: OffsetDateTime,
}

impl Blocks {
    pub async fn get_blocks_by_cid(
        transaction: &Database,
        normalized_cids: &[String],
    ) -> Result<Vec<Blocks>, sqlx::Error> {
        let mut prune_builder = sqlx::QueryBuilder::new("SELECT * FROM blocks WHERE cid IN(");

        let mut block_id_iterator = normalized_cids.iter().peekable();
        while let Some(bid) = block_id_iterator.next() {
            prune_builder.push_bind(bid);

            if block_id_iterator.peek().is_some() {
                prune_builder.push(", ");
            }
        }
        prune_builder.push(");");

        prune_builder
            .build_query_as::<Blocks>()
            .fetch_all(transaction)
            .await
    }

    pub async fn delete_blocks_by_cid(
        transaction: &mut DatabaseConnection,
        normalized_cids: &[String],
    ) -> Result<SqliteQueryResult, sqlx::Error> {
        let mut prune_builder = sqlx::QueryBuilder::new("DELETE FROM blocks WHERE cid IN(");

        let mut block_id_iterator = normalized_cids.iter().peekable();
        while let Some(bid) = block_id_iterator.next() {
            prune_builder.push_bind(bid);

            if block_id_iterator.peek().is_some() {
                prune_builder.push(", ");
            }
        }
        prune_builder.push(");");

        prune_builder.build().execute(&mut *transaction).await
    }
}
