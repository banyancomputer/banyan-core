use serde::{Serialize};
use time::OffsetDateTime;

use crate::{
    database::{
        DatabaseConnection,
    },
};

#[derive(sqlx::FromRow, Serialize)]
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
    pub async fn from_fingerprint(
        conn: &mut DatabaseConnection,
        fingerprint: &str,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            UserKey,
            r#"
                SELECT * FROM user_keys
                WHERE fingerprint = $2;
            "#,
            fingerprint,
        )
        .fetch_one(&mut *conn)
        .await
    }
}
