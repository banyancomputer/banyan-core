use time::OffsetDateTime;

use crate::database::{Database, DatabaseConnection};

/// Record a new reserved storage capacity authorization for a particular user. These are not made
/// active until a user redeems the signed ticket at the storage provider (the SP will call back to
/// this service indicating it was consumed).
pub struct NewStorageGrant<'a> {
    pub storage_host_id: &'a str,
    pub user_id: &'a str,

    /// The total amount of storage that will be reserved for the user at a particular storage
    /// host in bytes.
    pub authorized_amount: i64,
}

impl NewStorageGrant<'_> {
    pub async fn save(
        self,
        conn: &mut DatabaseConnection,
    ) -> Result<ExistingStorageGrant, sqlx::Error> {
        let created_at = OffsetDateTime::now_utc();
        let id: String = sqlx::query_scalar!(
            r#"INSERT INTO storage_grants (storage_host_id, user_id, authorized_amount, created_at)
                   VALUES ($1, $2, $3, $4)
                   RETURNING id;"#,
            self.storage_host_id,
            self.user_id,
            self.authorized_amount,
            created_at,
        )
        .fetch_one(&mut *conn)
        .await?;

        Ok(ExistingStorageGrant {
            id,
            storage_host_id: self.storage_host_id.to_string(),
            user_id: self.user_id.to_string(),
            authorized_amount: self.authorized_amount,
            created_at,
            redeemed_at: None,
        })
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ExistingStorageGrant {
    pub id: String,
    pub storage_host_id: String,
    pub user_id: String,
    pub authorized_amount: i64,
    pub created_at: OffsetDateTime,
    pub redeemed_at: Option<OffsetDateTime>,
}

impl ExistingStorageGrant {
    pub async fn latest_grant_for_host(
        conn: &Database,
        storage_host_id: &str,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            ExistingStorageGrant,
            r#"SELECT * FROM storage_grants WHERE storage_host_id = $1 ORDER BY created_at DESC LIMIT 1;"#,
            storage_host_id
        )
        .fetch_one(conn)
        .await
    }

    pub async fn redeem_storage_grant(
        conn: &mut DatabaseConnection,
        provider_id: &str,
        authorization_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"UPDATE storage_grants
               SET redeemed_at = CURRENT_TIMESTAMP
               WHERE storage_host_id = $1
                   AND id = $2
                   AND redeemed_at IS NULL;"#,
            provider_id,
            authorization_id,
        )
        .execute(&mut *conn)
        .await?;

        Ok(())
    }
    pub async fn find_by_host_and_user(
        con: &mut DatabaseConnection,
        storage_host_id: &str,
        user_id: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            ExistingStorageGrant,
            r#"SELECT * FROM storage_grants WHERE storage_host_id = $1 AND user_id = $2;"#,
            storage_host_id,
            user_id
        )
        .fetch_optional(&mut *con)
        .await
    }
}

#[cfg(test)]
mod tests {
    use crate::database::models::ExistingStorageGrant;
    use crate::database::test_helpers::{
        create_storage_grant, create_storage_hosts, sample_user, setup_database,
    };
    use crate::database::DatabaseConnection;
    impl ExistingStorageGrant {
        pub async fn find_by_id(
            conn: &mut DatabaseConnection,
            id: &str,
        ) -> Result<Self, sqlx::Error> {
            sqlx::query_as!(
                ExistingStorageGrant,
                r#"SELECT * FROM storage_grants WHERE id = $1;"#,
                id
            )
            .fetch_one(&mut *conn)
            .await
        }
    }
    #[tokio::test]
    async fn test_redeem_grant_works() {
        let db = setup_database().await;
        let mut conn = db.acquire().await.expect("connection");
        let provider_id = create_storage_hosts(&mut conn, "url1", "staging-service").await;
        let user_id = sample_user(&mut conn, "test@example.com").await;
        let authorization_id =
            create_storage_grant(&mut conn, provider_id.as_str(), &user_id, 100).await;

        let result =
            ExistingStorageGrant::redeem_storage_grant(&mut conn, &provider_id, &authorization_id)
                .await;
        assert!(result.is_ok());

        let grant = ExistingStorageGrant::find_by_id(&mut conn, &authorization_id)
            .await
            .expect("authorization grant");
        assert!(grant.redeemed_at.is_some());
    }
}
