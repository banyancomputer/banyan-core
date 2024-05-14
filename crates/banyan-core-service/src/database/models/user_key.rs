use serde::Serialize;
use time::OffsetDateTime;

use crate::database::DatabaseConnection;

#[derive(sqlx::FromRow, Debug, Serialize)]
pub struct UserKey {
    pub id: String,
    pub name: String,
    pub user_id: String,

    pub api_access: bool,

    pub pem: String,
    pub fingerprint: String,

    pub updated_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

impl UserKey {
    /// Create key
    pub async fn create(
        conn: &mut DatabaseConnection,
        name: &str,
        user_id: &str,
        fingerprint: &str,
        public_key: &str,
        api_access: bool,
    ) -> Result<String, sqlx::Error> {
        sqlx::query_scalar!(
            r#"
                INSERT INTO user_keys (name, user_id, fingerprint, pem, api_access)
                VALUES ($1, $2, $3, $4, $5)
                RETURNING id;
            "#,
            name,
            user_id,
            fingerprint,
            public_key,
            api_access,
        )
        .fetch_one(&mut *conn)
        .await
    }

    /// I think this might come in handy later but we're not using rn
    #[allow(dead_code)]
    pub async fn by_id(conn: &mut DatabaseConnection, id: &str) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            UserKey,
            r#"
                SELECT * FROM user_keys
                WHERE id = $1;
            "#,
            id,
        )
        .fetch_one(&mut *conn)
        .await
    }

    pub async fn by_fingerprint(
        conn: &mut DatabaseConnection,
        fingerprint: &str,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            UserKey,
            r#"
                SELECT * FROM user_keys
                WHERE fingerprint = $1;
            "#,
            fingerprint,
        )
        .fetch_one(&mut *conn)
        .await
    }
}
