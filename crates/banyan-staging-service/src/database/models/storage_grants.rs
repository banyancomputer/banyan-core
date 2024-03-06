use crate::database::{Database, DatabaseConnection};

#[derive(sqlx::FromRow)]
pub struct AuthorizedStorage {
    pub grant_id: String,
    pub allowed_bytes: i64,
}
impl AuthorizedStorage {
    pub async fn save(
        db: &Database,
        client_id: String,
        grant_id: String,
        allowed_storage: i64,
    ) -> Result<String, sqlx::Error> {
        sqlx::query_scalar!(
            "INSERT INTO storage_grants (client_id, grant_id, allowed_storage)
            VALUES ($1, $2, $3)
            RETURNING id;",
            client_id,
            grant_id,
            allowed_storage,
        )
        .fetch_one(db)
        .await
    }

    pub async fn save_in_transaction(
        conn: &mut DatabaseConnection,
        client_id: String,
        grant_id: String,
        allowed_storage: i64,
    ) -> Result<String, sqlx::Error> {
        sqlx::query_scalar!(
            "INSERT INTO storage_grants (client_id, grant_id, allowed_storage)
            VALUES ($1, $2, $3)
            RETURNING id;",
            client_id,
            grant_id,
            allowed_storage,
        )
        .fetch_one(&mut *conn)
        .await
    }

    pub async fn delete_by_grant_id(
        db: &mut DatabaseConnection,
        grant_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM storage_grants WHERE grant_id = $1;", grant_id)
            .execute(&mut *db)
            .await?;
        Ok(())
    }
    pub async fn current_authorized_storage(
        db: &Database,
        client_id: &str,
    ) -> Result<Option<AuthorizedStorage>, sqlx::Error> {
        let auth_stor = sqlx::query_as(
            "SELECT grant_id, allowed_storage AS allowed_bytes FROM storage_grants
                     WHERE client_id = $1
                     ORDER BY created_at DESC
                     LIMIT 1;",
        )
        .bind(client_id)
        .fetch_optional(db)
        .await?;

        Ok(auth_stor)
    }

    pub async fn get_client_by_grant_id(
        conn: &mut DatabaseConnection,
        grant_id: &str,
    ) -> Result<String, sqlx::Error> {
        let res = sqlx::query_scalar!(
            "SELECT client_id FROM storage_grants WHERE grant_id = $1;",
            grant_id,
        )
        .fetch_one(&mut *conn)
        .await?;

        Ok(res)
    }
}
