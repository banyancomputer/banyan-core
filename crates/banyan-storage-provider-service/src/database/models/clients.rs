use time::OffsetDateTime;

use crate::database::Database;

#[derive(Debug, sqlx::FromRow)]
pub struct Client {
    pub id: String,
    pub platform_id: String,
    pub fingerprint: String,
    pub public_key: String,
    pub created_at: OffsetDateTime,
}

impl Client {
    pub async fn find_by_fingerprint(
        conn: &Database,
        fingerprint: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        let client = sqlx::query_as!(
            Client,
            "SELECT * FROM clients WHERE fingerprint = $1",
            fingerprint
        )
        .fetch_optional(conn)
        .await?;

        Ok(client)
    }
}
