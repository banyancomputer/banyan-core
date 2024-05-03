use serde::{Deserialize, Serialize};

use crate::database::DatabaseConnection;

#[derive(Debug, sqlx::FromRow, Serialize, Deserialize)]
pub struct Clients {
    pub id: String,
    pub platform_id: String,
    pub fingerprint: String,
    pub public_key: String,
}

impl Clients {
    pub async fn find_by_upload_id(
        conn: &mut DatabaseConnection,
        upload_id: &str,
    ) -> Result<Self, sqlx::Error> {
        let client = sqlx::query_as!(
            Clients,
            "SELECT c.id, c.platform_id, c.fingerprint, c.public_key FROM clients AS c
              JOIN uploads u on c.id = u.client_id
            WHERE u.id = $1;",
            upload_id
        )
        .fetch_one(conn)
        .await?;
        Ok(client)
    }
}
