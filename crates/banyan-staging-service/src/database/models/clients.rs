use time::OffsetDateTime;

use crate::database::Database;

#[derive(Debug, sqlx::FromRow)]
pub struct Clients {
    pub id: String,
    pub platform_id: String,
    pub fingerprint: String,
    pub public_key: String,
    pub created_at: OffsetDateTime,
}

impl Clients {
    pub async fn find_by_upload_id(conn: &Database, upload_id: &str) -> Result<Self, sqlx::Error> {
        let client = sqlx::query_as!(
            Clients,
            "SELECT c.* FROM clients AS c
                INNER JOIN uploads u on c.id = u.client_id
            WHERE u.id = $1;",
            upload_id
        )
        .fetch_one(conn)
        .await?;
        Ok(client)
    }
}
