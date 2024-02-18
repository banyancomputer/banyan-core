use crate::database::Database;

#[derive(sqlx::FromRow)]
pub struct AuthorizedStorage {
    pub id: String,
    pub grant_id: String,
    pub allowed_bytes: i64,
}

impl AuthorizedStorage {
    pub async fn current_authorized_storage(
        db: &Database,
        client_id: &str,
    ) -> Result<Option<AuthorizedStorage>, sqlx::Error> {
        sqlx::query_as(
            "SELECT id, grant_id, allowed_storage AS allowed_bytes FROM storage_grants
                     WHERE client_id = $1
                     ORDER BY created_at DESC
                     LIMIT 1;",
        )
        .bind(client_id)
        .fetch_optional(db)
        .await
    }
    pub async fn create(
        db: &Database,
        client_id: &str,
        grant_id: &str,
        allowed_bytes: i64,
    ) -> Result<String, sqlx::Error> {
        sqlx::query_scalar!(
            r#"INSERT INTO storage_grants (grant_id, client_id, allowed_storage)
                    VALUES ($1, $2, $3)
                    RETURNING id;"#,
            grant_id,
            client_id,
            allowed_bytes,
        )
        .fetch_one(db)
        .await
    }

    pub async fn find_by_client_and_grant(
        db: &Database,
        client_id: &str,
        grant_id: &str,
    ) -> Result<Option<String>, sqlx::Error> {
        let res = sqlx::query_scalar!(
            "SELECT id AS allowed_bytes FROM storage_grants
                     WHERE client_id = $1 AND grant_id = $2;",
            client_id,
            grant_id,
        )
        .fetch_optional(db)
        .await?;
        Ok(res)
    }
    pub async fn create_if_missing(
        db: &Database,
        client_id: &str,
        grant_id: &str,
        allowed_bytes: i64,
    ) -> Result<String, sqlx::Error> {
        let storage_grant_id = Self::find_by_client_and_grant(&db, &client_id, grant_id).await?;
        let storage_grant_id = match storage_grant_id {
            Some(storage_grant_id) => storage_grant_id,
            None => Self::create(&db, &client_id, &grant_id, allowed_bytes).await?,
        };

        Ok(storage_grant_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::test_helpers::{create_client, create_storage_grant, setup_database};

    #[tokio::test]
    async fn current_storage_returned_successfully() {
        let db = setup_database().await;
        let client_id =
            create_client(&db, "test_platform", "test_fingerprint", "test_public_key").await;
        create_storage_grant(&db, client_id.as_str(), "test_grant", 1000).await;

        let result = AuthorizedStorage::current_authorized_storage(&db, client_id.as_str()).await;

        assert!(result.is_ok());
        let authorized_storage = result.unwrap().expect("authorized storage");
        assert_eq!(authorized_storage.grant_id, "test_grant");
        assert_eq!(authorized_storage.allowed_bytes, 1000);
    }

    #[tokio::test]
    async fn missing_storage_not_returned() {
        let db = setup_database().await;
        let missing_client_id = "missing_client";
        let result = AuthorizedStorage::current_authorized_storage(&db, missing_client_id).await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn create_if_missing_creates_one_entry_for_same_grant_and_client() {
        let db = setup_database().await;
        let test_grant = "test_grant";
        let allowed_bytes = 1000;
        let client_id =
            create_client(&db, "test_platform", "test_fingerprint", "test_public_key").await;

        let old_storage_id =
            AuthorizedStorage::create_if_missing(&db, &client_id, test_grant, allowed_bytes).await;
        assert!(old_storage_id.is_ok());

        let new_storage_id =
            AuthorizedStorage::create_if_missing(&db, &client_id, test_grant, allowed_bytes).await;
        assert!(new_storage_id.is_ok());
        assert_eq!(new_storage_id.unwrap(), old_storage_id.unwrap());
    }
}
