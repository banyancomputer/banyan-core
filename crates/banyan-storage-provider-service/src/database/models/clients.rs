use crate::database::Database;

#[derive(Debug, sqlx::FromRow)]
pub struct Clients {
    pub id: String,
    pub platform_id: String,
    pub fingerprint: String,
    pub public_key: String,
}

impl Clients {
    pub async fn find_bu_metadata_id(
        conn: &Database,
        metadata_id: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Clients,
            "SELECT c.id, c.platform_id, c.fingerprint, c.public_key FROM clients AS c
              JOIN uploads u on c.id = u.client_id
            WHERE u.metadata_id = $1;",
            metadata_id
        )
        .fetch_optional(conn)
        .await
    }

    pub async fn find_by_upload_id(conn: &Database, upload_id: &str) -> Result<Self, sqlx::Error> {
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
    pub async fn create_if_missing(
        conn: &Database,
        platform_id: &str,
        fingerprint: &str,
        public_key: &str,
    ) -> Result<String, sqlx::Error> {
        let client = Self::find_by_fingerprint(conn, fingerprint).await?;
        let client_id = match client {
            Some(client) => client.id,
            None => Self::create(conn, platform_id, fingerprint, public_key).await?,
        };

        Ok(client_id)
    }
    pub async fn find_by_fingerprint(
        conn: &Database,
        fingerprint: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        let client = sqlx::query_as!(
            Clients,
            "SELECT id, platform_id, fingerprint, public_key FROM clients WHERE fingerprint = $1;",
            fingerprint
        )
        .fetch_optional(conn)
        .await?;
        Ok(client)
    }
    pub async fn create(
        conn: &Database,
        platform_id: &str,
        fingerprint: &str,
        public_key: &str,
    ) -> Result<String, sqlx::Error> {
        let client_id = sqlx::query_scalar!(
            "INSERT INTO clients (platform_id, fingerprint, public_key) VALUES ($1, $2, $3) RETURNING id;",
            platform_id,
            fingerprint,
            public_key
        )
        .fetch_one(conn)
        .await?;
        Ok(client_id)
    }
}
