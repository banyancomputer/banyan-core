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

    //
    pub updated_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

impl UserKey {
    pub async fn by_id(conn: &mut DatabaseConnection, id: &str) -> Result<Self, sqlx::Error> {
        tracing::warn!("looking key up by id {id}");

        let all_keys: Vec<UserKey> = sqlx::query_as!(
            UserKey,
            r#"
                SELECT * FROM user_keys;
            "#,
        )
        .fetch_all(&mut *conn)
        .await?;
        tracing::warn!("all_keys: {:?}", all_keys);
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
        tracing::warn!("looking key up by fingerprint {fingerprint}");
        let all_keys: Vec<UserKey> = sqlx::query_as!(
            UserKey,
            r#"
                SELECT * FROM user_keys;
            "#,
        )
        .fetch_all(&mut *conn)
        .await?;
        tracing::warn!("all_keys: {:?}", all_keys);
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
