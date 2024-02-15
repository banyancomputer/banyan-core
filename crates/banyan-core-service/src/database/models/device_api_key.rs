use jwt_simple::prelude::Serialize;

use crate::database::Database;

#[derive(sqlx::FromRow, Serialize)]
pub struct DeviceApiKey {
    id: String,
    user_id: String,
    fingerprint: String,
    pem: String,
}

impl DeviceApiKey {
    pub async fn get_keys_for_user(
        database: &Database,
        user_id: &str,
    ) -> Result<Vec<Self>, sqlx::Error> {
        sqlx::query_as!(
            DeviceApiKey,
            r#"SELECT id, user_id, fingerprint, pem FROM device_api_keys WHERE user_id = $1;"#,
            user_id,
        )
        .fetch_all(database)
        .await
    }

    pub async fn get_by_id_and_user(
        database: &Database,
        key_id: &str,
        user_id: &str,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            DeviceApiKey,
            r#"SELECT id, user_id, fingerprint, pem
               FROM device_api_keys
               WHERE id = $1 AND user_id = $2;"#,
            key_id,
            user_id,
        )
        .fetch_one(database)
        .await
    }
}
