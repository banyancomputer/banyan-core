use sqlx::sqlite::SqliteQueryResult;
use time::OffsetDateTime;

use crate::database::{Database, DatabaseConnection};

#[derive(sqlx::FromRow)]
pub struct Uploads {
    pub id: String,
    pub client_id: String,
    pub metadata_id: String,
    pub base_path: String,
    pub reported_size: i64,
    pub final_size: Option<i64>,
    pub state: String,
    pub integrity_hash: Option<String>,
    pub started_at: OffsetDateTime,
    pub created_at: Option<OffsetDateTime>,
    pub finished_at: Option<OffsetDateTime>,
}

impl Uploads {
    pub async fn get_by_metadata_id(
        pool: &Database,
        metadata_id: &str,
    ) -> sqlx::Result<Self, sqlx::Error> {
        sqlx::query_as!(
            Self,
            "SELECT * FROM uploads WHERE metadata_id = $1",
            metadata_id
        )
        .fetch_one(pool)
        .await
    }
    pub async fn delete_by_metadata_id(
        transaction: &mut DatabaseConnection,
        metadata_id: &str,
    ) -> sqlx::Result<SqliteQueryResult, sqlx::Error> {
        sqlx::query!("DELETE FROM uploads WHERE metadata_id = $1", metadata_id,)
            .execute(transaction)
            .await
    }
}
