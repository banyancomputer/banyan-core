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
