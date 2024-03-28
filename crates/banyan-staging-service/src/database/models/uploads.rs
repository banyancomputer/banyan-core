use sqlx::sqlite::SqliteQueryResult;
use time::OffsetDateTime;

use crate::database::{Database, DatabaseConnection};

pub struct CreateUpload<'a> {
    pub(crate) client_id: &'a str,
    pub(crate) metadata_id: &'a str,
    pub(crate) reported_size: i64,
}

impl<'a> CreateUpload<'a> {
    pub async fn save(self, conn: &mut DatabaseConnection) -> Result<String, sqlx::Error> {
        let time = OffsetDateTime::now_utc();

        sqlx::query_scalar!(
            r#"INSERT INTO uploads
                 (client_id, metadata_id, reported_size, base_path, state, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6)
                 RETURNING id;"#,
            self.client_id,
            self.metadata_id,
            self.reported_size,
            self.metadata_id,
            "started",
            time
        )
        .fetch_one(&mut *conn)
        .await
    }
}

#[derive(sqlx::FromRow)]
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

    pub created_at: Option<OffsetDateTime>,
}

impl Uploads {
    pub async fn by_id_and_client(
        conn: &mut DatabaseConnection,
        upload_id: &str,
        client_id: &str,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            Uploads,
            r#"SELECT * FROM uploads WHERE id = $1 AND client_id = $2;"#,
            upload_id,
            client_id,
        )
        .fetch_one(&mut *conn)
        .await
    }

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
