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

#[derive(sqlx::FromRow, Debug)]
pub struct AuthorizedAmounts {
    pub authorized_amount: i64,
    pub storage_grant_id: String,
    pub storage_host_name: String,
    pub storage_host_url: String,
}

impl AuthorizedAmounts {
    pub async fn lookup(
        conn: &mut DatabaseConnection,
        user_id: &str,
        bucket_id: &str,
    ) -> Result<Vec<AuthorizedAmounts>, sqlx::Error> {
        sqlx::query_as!(
            AuthorizedAmounts,
            r#"WITH current_grants AS (
                SELECT id, storage_host_id, user_id, MAX(redeemed_at) AS most_recently_redeemed_at
                FROM storage_grants
                WHERE redeemed_at IS NOT NULL AND user_id = $1
                GROUP BY storage_host_id, user_id
            )
                SELECT sg.id AS storage_grant_id, sg.authorized_amount, sh.name AS storage_host_name, sh.url AS storage_host_url
                    FROM current_grants AS cg
                    JOIN storage_hosts_metadatas_storage_grants AS shms ON shms.storage_grant_id = cg.id
                    JOIN storage_hosts AS sh ON sh.id = shms.storage_host_id
                    JOIN metadata AS m ON m.id = shms.metadata_id
                    JOIN buckets AS b ON b.id = m.bucket_id
                    JOIN storage_grants AS sg ON sg.id = cg.id
                    WHERE b.user_id = $1
                        AND b.id = $2
                        AND b.deleted_at IS NULL
                        AND m.state NOT IN ('deleted', 'upload_failed');"#,
                        // TODO: Are we not looking ^ just for 'current' metadata
            user_id,
            bucket_id,
        )
            .fetch_all(conn)
            .await
    }
}
