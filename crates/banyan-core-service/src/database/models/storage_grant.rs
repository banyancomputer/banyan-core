use time::OffsetDateTime;

use crate::database::DatabaseConnection;

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
    pub async fn save(self, conn: &mut DatabaseConnection) -> Result<String, sqlx::Error> {
        sqlx::query_scalar!(
            r#"INSERT INTO storage_grants (storage_host_id, user_id, authorized_amount)
                   VALUES ($1, $2, $3)
                   RETURNING id;"#,
            self.storage_host_id,
            self.user_id,
            self.authorized_amount,
        )
        .fetch_one(&mut *conn)
        .await
    }
}

#[derive(sqlx::FromRow)]
pub struct ExistingStorageGrant {
    pub id: String,
    pub storage_host_id: String,
    pub user_id: String,
    pub authorized_amount: i64,
    pub created_at: OffsetDateTime,
    pub redeemed_at: Option<OffsetDateTime>,
}

impl ExistingStorageGrant {
    pub async fn find_by_id(conn: &mut DatabaseConnection, id: &str) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            ExistingStorageGrant,
            r#"SELECT * FROM storage_grants WHERE id = $1;"#,
            id
        )
        .fetch_one(&mut *conn)
        .await
    }

    pub async fn update_storage_host_for_grant(
        conn: &mut DatabaseConnection,
        id: &str,
        storage_host_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"UPDATE storage_grants SET storage_host_id = $1 WHERE id = $2;"#,
            storage_host_id,
            id
        )
        .execute(&mut *conn)
        .await
        .map(|_| ())
    }
}
