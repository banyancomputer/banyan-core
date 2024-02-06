use time::OffsetDateTime;

use crate::database::Database;

#[derive(Debug, sqlx::FromRow)]
pub struct Uploads {
    pub id: String,
    pub client_id: String,
    pub metadata_id: String,
    pub reported_size: i64,
    pub final_size: Option<i64>,
    pub base_path: String,
    pub state: String,
    pub integrity_hash: Option<String>,
    pub started_at: OffsetDateTime,
    pub finished_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
}

impl Uploads {
    pub async fn non_pruned_uploads(database: &Database) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            Uploads,
            "SELECT u.*
                FROM uploads AS u
                 JOIN uploads_blocks AS ub ON ub.upload_id = u.id
                where pruned_at IS NULL;"
        )
        .fetch_all(database)
        .await
    }

    pub async fn get_by_id(database: &Database, id: &str) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(Uploads, "SELECT * FROM uploads WHERE id = $1;", id)
            .fetch_optional(database)
            .await
    }
}
