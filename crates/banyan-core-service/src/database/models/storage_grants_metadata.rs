use crate::database::{Database, DatabaseConnection};

#[derive(sqlx::FromRow)]
pub struct StorageHostsMetadatasStorageGrants {
    pub storage_host_id: String,
    pub metadata_id: String,
    pub storage_grant_id: String,
}

impl StorageHostsMetadatasStorageGrants {
    pub async fn find_by_metadata_id(
        conn: &Database,
        metadata_id: &str,
    ) -> Result<Self, sqlx::Error> {
        let storage_host = sqlx::query_as!(
            StorageHostsMetadatasStorageGrants,
            "SELECT * FROM storage_hosts_metadatas_storage_grants WHERE metadata_id = $1;",
            metadata_id
        )
        .fetch_one(conn)
        .await?;
        Ok(storage_host)
    }

    pub async fn update_host_by_metadata(
        conn: &mut DatabaseConnection,
        metadata_id: &str,
        storage_host_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE storage_hosts_metadatas_storage_grants SET storage_host_id = $1 WHERE metadata_id = $2;",
            storage_host_id,
            metadata_id
        )
        .execute(&mut *conn)
        .await?;
        Ok(())
    }
}
