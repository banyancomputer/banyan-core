use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use banyan_task::TaskLikeExt;
use serde::Deserialize;

use crate::app::AppState;
use crate::database::models::{
    Blocks, ExistingStorageGrant, Metadata, MinimalBlockLocation, StorageHost,
    StorageHostsMetadatasStorageGrants,
};
use crate::extractors::StorageProviderIdentity;
use crate::tasks::{DeleteStagingDataTask, HostCapacityTask};

#[derive(Deserialize)]
pub struct CompleteRedistributionRequest {
    replication: bool,
    normalized_cids: Vec<String>,
    grant_id: String,
}

pub async fn handler(
    storage_provider: StorageProviderIdentity,
    State(state): State<AppState>,
    Path(metadata_id): Path<String>,
    Json(request): Json<CompleteRedistributionRequest>,
) -> Result<Response, CompleteRedistributionError> {
    let database = state.database();
    // validate the metadata exists
    Metadata::find_by_id(&database, &metadata_id.to_string()).await?;

    let new_storage_host_id = storage_provider.id.clone();
    let staging_host = StorageHost::select_staging(&database).await?;

    let mut transaction = database.begin().await?;

    ExistingStorageGrant::redeem_storage_grant(
        &mut transaction,
        &new_storage_host_id,
        &request.grant_id,
    )
    .await
    .map_err(CompleteRedistributionError::RedeemFailed)?;

    StorageHostsMetadatasStorageGrants::associate_upload(
        &mut transaction,
        &new_storage_host_id,
        &metadata_id,
        &request.grant_id,
    )
    .await?;

    let block_ids = Blocks::get_ids_by_cids(&mut transaction, &request.normalized_cids).await?;
    if block_ids.len() != request.normalized_cids.len() {
        return Err(CompleteRedistributionError::MissingBlocks(format!(
            "not enough blocks found {} for cids {} for metadata {} from host {} to host {}",
            block_ids.len(),
            request.normalized_cids.len(),
            metadata_id,
            staging_host.id,
            new_storage_host_id
        )));
    }

    if !request.replication {
        let deleted_blocks: u64 = MinimalBlockLocation::delete_blocks_for_host(
            &mut transaction,
            &block_ids,
            &staging_host.id,
        )
        .await?
        .iter()
        .map(|r| r.rows_affected())
        .sum();

        // deleting more blocks is fine, since (although rare) there are cases of block duplication
        // between uploads (thus between metadata_ids). Those duplicated blocks will not be
        // added to the blocks table, but they will be added to the block_locations table
        // when it's a replication there is nothing to delete from the staging host
        if deleted_blocks < block_ids.len() as u64 {
            return Err(CompleteRedistributionError::UpdateFailed(format!(
                "deleted {} vs cids {} for metadata {} from host {}",
                deleted_blocks,
                block_ids.len(),
                metadata_id,
                staging_host.id,
            )));
        }
    }

    let updated_blocks =
        MinimalBlockLocation::update_stored_at(&mut transaction, &block_ids, &new_storage_host_id)
            .await?;
    if updated_blocks
        .iter()
        .map(|r| r.rows_affected())
        .sum::<u64>()
        != block_ids.len() as u64
    {
        return Err(CompleteRedistributionError::UpdateFailed(format!(
            "updated {} vs passed cids {} for metadata {} from host {}",
            updated_blocks
                .iter()
                .map(|r| r.rows_affected())
                .sum::<u64>(),
            block_ids.len(),
            metadata_id,
            staging_host.id,
        )));
    }

    // Now, let's re-evaluate the capacity of the new storage host
    HostCapacityTask::new(new_storage_host_id)
        .enqueue::<banyan_task::SqliteTaskStore>(&mut *transaction)
        .await
        .map_err(CompleteRedistributionError::UnableToEnqueueTask)?;

    if !request.replication {
        //  revaluate the capacity of staging service
        HostCapacityTask::new(staging_host.id)
            .enqueue::<banyan_task::SqliteTaskStore>(&mut transaction)
            .await
            .map_err(CompleteRedistributionError::UnableToEnqueueTask)?;
        DeleteStagingDataTask::new(metadata_id, request.normalized_cids.clone())
            .enqueue::<banyan_task::SqliteTaskStore>(&mut *transaction)
            .await
            .map_err(CompleteRedistributionError::UnableToEnqueueTask)?;
    }

    transaction.commit().await?;

    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum CompleteRedistributionError {
    #[error("failed to register storage grant as redeemed: {0}")]
    RedeemFailed(sqlx::Error),

    #[error("missing blocks from record: {0}")]
    MissingBlocks(String),

    #[error("failed to update the database correctly: {0}")]
    UpdateFailed(String),

    #[error("failed to run query against database: {0}")]
    QueryFailed(#[from] sqlx::Error),

    #[error("could not enqueue task: {0}")]
    UnableToEnqueueTask(banyan_task::TaskStoreError),
}

impl IntoResponse for CompleteRedistributionError {
    fn into_response(self) -> Response {
        tracing::error!("{self}");
        let err_msg = serde_json::json!({"msg": "internal server error"});
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
    }
}
#[cfg(test)]
mod tests {
    use axum::extract::Path;
    use axum::Json;
    use banyan_task::SqliteTaskStore;
    use http::StatusCode;

    use crate::app::mock_app_state;
    use crate::database::models::{Blocks, ExistingStorageGrant, MetadataState};
    use crate::database::{test_helpers, Database};
    use crate::extractors::StorageProviderIdentity;
    use crate::hooks::storage::complete_distribution::{handler, CompleteRedistributionRequest};
    use crate::tasks::DeleteStagingDataTask;

    pub async fn select_storage_grants_for_host(
        conn: &Database,
        storage_host_id: &str,
    ) -> Option<ExistingStorageGrant> {
        sqlx::query_as!(
            ExistingStorageGrant,
            "SELECT * FROM storage_grants WHERE storage_host_id = $1",
            storage_host_id
        )
        .fetch_optional(conn)
        .await
        .expect("storage grants")
    }

    pub async fn select_storage_metadata_grant_for_host(
        conn: &Database,
        storage_host_id: &str,
    ) -> Option<String> {
        sqlx::query_scalar!(
            "SELECT metadata_id FROM storage_hosts_metadatas_storage_grants WHERE storage_host_id = $1",
            storage_host_id
        )
            .fetch_optional(conn)
            .await
            .expect("storage semgnets")
    }

    pub async fn select_blocks_for_host(conn: &Database, storage_host_id: &str) -> Vec<String> {
        sqlx::query_scalar!(
            "SELECT block_id FROM block_locations WHERE storage_host_id = $1",
            storage_host_id
        )
        .fetch_all(conn)
        .await
        .expect("block cids")
    }

    async fn setup_test_environment() -> (Database, String, String, String, String, Vec<String>) {
        let db = test_helpers::setup_database().await;
        let mut conn = db.acquire().await.expect("connection");

        let staging_host_id = test_helpers::create_storage_host(
            &mut conn,
            "staging-service",
            "https://127.0.0.1:8001/",
            1_000_000,
        )
        .await;
        let new_storage_host_id = test_helpers::create_storage_host(
            &mut conn,
            "Bax",
            "https://127.0.0.1:8002/",
            3_000_000,
        )
        .await;
        let second_host_id = test_helpers::create_storage_host(
            &mut conn,
            "Second Host",
            "https://127.0.0.1:8003/",
            3_000_000,
        )
        .await;

        let user_id = test_helpers::sample_user(&mut conn, "user@domain.tld").await;
        let bucket_id = test_helpers::sample_bucket(&mut conn, &user_id).await;
        let storage_grant_id =
            test_helpers::create_storage_grant(&mut conn, &staging_host_id, &user_id, 1_000_000)
                .await;
        let metadata_id =
            test_helpers::sample_metadata(&mut conn, &bucket_id, 1, MetadataState::Current).await;
        let new_storage_grant_id = test_helpers::create_storage_grant(
            &mut conn,
            &new_storage_host_id,
            &user_id,
            1_000_000,
        )
        .await;
        let block_ids = test_helpers::sample_blocks(
            &mut conn,
            4,
            &metadata_id,
            &staging_host_id,
            &storage_grant_id,
        )
        .await;

        test_helpers::associate_blocks(
            &mut conn,
            &metadata_id,
            &new_storage_host_id,
            block_ids.iter().map(String::as_str),
        )
        .await;

        test_helpers::associate_blocks(
            &mut conn,
            &metadata_id,
            &second_host_id,
            block_ids.iter().map(String::as_str),
        )
        .await;

        let block_cids: Vec<String> = Blocks::get_cids_by_ids(&mut conn, &block_ids)
            .await
            .expect("cids");
        (
            db,
            new_storage_host_id,
            metadata_id,
            new_storage_grant_id.to_string(),
            staging_host_id,
            block_cids,
        )
    }

    #[tokio::test]
    async fn handler_returns_success_for_the_happy_case() {
        let (
            db,
            new_storage_host_id,
            metadata_id,
            new_storage_grant_id,
            staging_host_id,
            block_cids,
        ) = setup_test_environment().await;
        let res = handler(
            StorageProviderIdentity::default().with_host_id(&new_storage_host_id),
            mock_app_state(db.clone()),
            Path(metadata_id.clone()),
            Json(CompleteRedistributionRequest {
                replication: false,
                normalized_cids: block_cids.clone(),
                grant_id: new_storage_grant_id,
            }),
        )
        .await;

        assert!(res.is_ok());
        assert_eq!(res.unwrap().status(), StatusCode::NO_CONTENT);

        let old_host = select_blocks_for_host(&db, &staging_host_id).await;
        assert_eq!(old_host.len(), 0);
        let staging_grant_metadata =
            select_storage_metadata_grant_for_host(&db, &staging_host_id).await;
        assert!(staging_grant_metadata.is_some());
        let staging_grant = select_storage_grants_for_host(&db, &staging_host_id).await;
        assert!(staging_grant.is_some());

        let blocks_for_host = select_blocks_for_host(&db, &new_storage_host_id).await;
        assert_eq!(blocks_for_host.len(), block_cids.len());
        let new_storage_grant_metadata =
            select_storage_metadata_grant_for_host(&db, &new_storage_host_id).await;
        assert!(new_storage_grant_metadata.is_some());
        let new_storage_grant = select_storage_grants_for_host(&db, &new_storage_host_id).await;
        assert!(new_storage_grant.is_some());
        assert!(new_storage_grant.unwrap().redeemed_at.is_some());
        let mut conn = db.acquire().await.expect("database connection");
        assert!(SqliteTaskStore::is_present(
            &mut conn,
            &DeleteStagingDataTask::new(metadata_id, block_cids.clone())
        )
        .await
        .expect("could not retrieve task"));
    }

    #[tokio::test]
    async fn handler_rolls_back_on_update_blocks_failure() {
        let (
            db,
            new_storage_host_id,
            metadata_id,
            new_storage_grant_id,
            staging_host_id,
            block_cids,
        ) = setup_test_environment().await;

        // Simulate a failure in update_blocks
        let res = handler(
            StorageProviderIdentity::default().with_host_id(&new_storage_host_id),
            mock_app_state(db.clone()),
            Path(metadata_id.clone()),
            Json(CompleteRedistributionRequest {
                replication: false,
                normalized_cids: vec!["fake-cid".to_string()],
                grant_id: new_storage_grant_id,
            }),
        )
        .await;

        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), format!(
            "missing blocks from record: not enough blocks found {} for cids {} for metadata {} from host {} to host {}",
            0,
            1,
            metadata_id,
            staging_host_id,
            new_storage_host_id
        ));
        let old_host = select_blocks_for_host(&db, &staging_host_id).await;
        assert_eq!(old_host.len(), block_cids.len());
        let storage_metadata = select_storage_metadata_grant_for_host(&db, &staging_host_id).await;
        assert!(storage_metadata.is_some());
        let storage_grant = select_storage_grants_for_host(&db, &staging_host_id).await;
        assert!(storage_grant.is_some());

        let storage_metadata =
            select_storage_metadata_grant_for_host(&db, &new_storage_host_id).await;
        assert!(storage_metadata.is_none());
        let storage_grant = select_storage_grants_for_host(&db, &new_storage_host_id).await;
        assert!(storage_grant.is_some());
        assert!(storage_grant.unwrap().redeemed_at.is_none());
    }

    #[tokio::test]
    async fn handler_does_not_delete_blocks_on_replication() {
        let (
            db,
            new_storage_host_id,
            metadata_id,
            new_storage_grant_id,
            staging_host_id,
            block_cids,
        ) = setup_test_environment().await;

        let res = handler(
            StorageProviderIdentity::default().with_host_id(&new_storage_host_id),
            mock_app_state(db.clone()),
            Path(metadata_id.clone()),
            Json(CompleteRedistributionRequest {
                replication: true,
                normalized_cids: block_cids.clone(),
                grant_id: new_storage_grant_id,
            }),
        )
        .await;

        assert!(res.is_ok());
        assert_eq!(res.unwrap().status(), StatusCode::NO_CONTENT);

        let old_host = select_blocks_for_host(&db, &staging_host_id).await;
        assert_eq!(old_host.len(), block_cids.len());
        let staging_grant_metadata =
            select_storage_metadata_grant_for_host(&db, &staging_host_id).await;
        assert!(staging_grant_metadata.is_some());
        let staging_grant = select_storage_grants_for_host(&db, &staging_host_id).await;
        assert!(staging_grant.is_some());

        let blocks_for_host = select_blocks_for_host(&db, &new_storage_host_id).await;
        assert_eq!(blocks_for_host.len(), block_cids.len());
        let new_storage_grant_metadata =
            select_storage_metadata_grant_for_host(&db, &new_storage_host_id).await;
        assert!(new_storage_grant_metadata.is_some());
        let new_storage_grant = select_storage_grants_for_host(&db, &new_storage_host_id).await;
        assert!(new_storage_grant.is_some());
        assert!(new_storage_grant.unwrap().redeemed_at.is_some());

        let mut conn = db.acquire().await.expect("database connection");
        assert!(!SqliteTaskStore::is_present(
            &mut conn,
            &DeleteStagingDataTask::new(metadata_id, block_cids.clone())
        )
        .await
        .expect("could not retrieve task"));
    }
}
