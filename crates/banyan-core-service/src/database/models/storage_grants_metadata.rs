use crate::database::{Database, DatabaseConnection};

#[derive(sqlx::FromRow)]
pub struct StorageHostsMetadatasStorageGrants {
    pub storage_host_id: String,
    pub metadata_id: String,
    pub storage_grant_id: String,
}

impl StorageHostsMetadatasStorageGrants {
    pub async fn find_by_metadata_and_storage_host(
        conn: &Database,
        metadata_id: &str,
        storage_host_id: &str,
    ) -> Result<StorageHostsMetadatasStorageGrants, sqlx::Error> {
        sqlx::query_as!(
            StorageHostsMetadatasStorageGrants,
            r#"SELECT * FROM storage_hosts_metadatas_storage_grants WHERE storage_host_id = $1 AND metadata_id = $2;"#,
            storage_host_id,
            metadata_id
        )
        .fetch_one(conn)
        .await
    }

    pub async fn associate_upload(
        conn: &mut DatabaseConnection,
        provider_id: &str,
        metadata_id: &str,
        authorization_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"INSERT OR IGNORE INTO storage_hosts_metadatas_storage_grants
               (storage_host_id, metadata_id, storage_grant_id)
               VALUES ($1, $2, $3);"#,
            provider_id,
            metadata_id,
            authorization_id,
        )
        .execute(&mut *conn)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::database::models::{MetadataState, StorageHostsMetadatasStorageGrants};
    use crate::database::test_helpers::{
        create_storage_grant, create_storage_hosts, sample_bucket, sample_metadata, sample_user,
        setup_database,
    };

    #[tokio::test]
    async fn test_associate_upload() {
        let db = setup_database().await;
        let mut conn = db.acquire().await.expect("connection");
        let provider_id = create_storage_hosts(&mut conn, "url1", "staging").await;
        let user_id = sample_user(&mut conn, "test@example.com").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;
        let metadata_id = sample_metadata(&mut conn, &bucket_id, 1, MetadataState::Current).await;
        let authorization_id =
            create_storage_grant(&mut conn, provider_id.as_str(), &user_id, 100).await;

        let result = StorageHostsMetadatasStorageGrants::associate_upload(
            &mut conn,
            &provider_id,
            &metadata_id,
            &authorization_id,
        )
        .await;
        assert!(result.is_ok());
        let result = StorageHostsMetadatasStorageGrants::find_by_metadata_and_storage_host(
            &db,
            &metadata_id,
            &provider_id,
        )
        .await;
        assert!(result.is_ok());
        let storage_grant_metadata = result.unwrap();
        assert_eq!(storage_grant_metadata.storage_host_id, provider_id);
        assert_eq!(storage_grant_metadata.metadata_id, metadata_id);
        assert_eq!(storage_grant_metadata.storage_grant_id, authorization_id);
    }
}
