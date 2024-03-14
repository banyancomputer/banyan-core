use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use banyan_task::TaskLikeExt;
use serde::Deserialize;
use uuid::Uuid;

use crate::app::AppState;
use crate::database::models::{
    Blocks, ExistingStorageGrant, Metadata, MinimalBlockLocation,
    StorageHostsMetadatasStorageGrants,
};
use crate::extractors::StorageProviderIdentity;
use crate::tasks::{HostCapacityTask, ReportStorageHostConsumptionTask, ReportUserConsumptionTask};

/// When a client finishes uploading their data to either staging or a storage host, the storage
/// host will make a request to this end point letting us know that we have all the data safely
/// stored and can mark the associated metadata as ready to be consumed by downstream clients.
pub async fn handler(
    storage_provider: StorageProviderIdentity,
    State(state): State<AppState>,
    Path(metadata_id): Path<Uuid>,
    Json(request): Json<ReportUploadRequest>,
) -> Result<Response, ReportUploadError> {
    let db_metadata_id = metadata_id.to_string();

    let mut database = state.database();
    let mut db_conn = database.acquire().await?;

    ExistingStorageGrant::redeem_storage_grant(
        &mut db_conn,
        &storage_provider.id,
        &request.storage_authorization_id,
    )
    .await
    .map_err(ReportUploadError::RedeemFailed)?;

    StorageHostsMetadatasStorageGrants::associate_upload(
        &mut db_conn,
        &storage_provider.id,
        &db_metadata_id,
        &request.storage_authorization_id,
    )
    .await
    .map_err(ReportUploadError::NoUploadAssociation)?;

    for block_cid in request.normalized_cids.iter() {
        Blocks::insert_block_cid(&mut db_conn, block_cid)
            .await
            .map_err(ReportUploadError::UnableToRecordBlock)?;
        let block_id = Blocks::get_block_id(&mut db_conn, block_cid)
            .await
            .map_err(ReportUploadError::UnableToRecordBlock)?;
        MinimalBlockLocation {
            block_id,
            metadata_id: db_metadata_id.clone(),
            storage_host_id: storage_provider.id.clone(),
        }
        .save_with_stored_at(&mut db_conn)
        .await
        .map_err(ReportUploadError::UnableToRecordBlock)?;
    }

    let bucket_id = Metadata::get_bucket_id(&mut db_conn, &db_metadata_id)
        .await
        .map_err(ReportUploadError::MarkCurrentFailed)?;

    // TODO: if a storage host is reporting for a piece of metadata in a bucket that has since
    // been soft-deleted, then this will return an error.
    // We should handle that invariant more explicitly.
    Metadata::mark_current(
        &mut db_conn,
        &bucket_id,
        &db_metadata_id,
        Some(request.data_size),
    )
    .await
    .map_err(ReportUploadError::MarkCurrentFailed)?;
    // Now that the state has changed, mark old unsnapshotted metadatas as being deleted
    Metadata::delete_outdated(&mut db_conn, &bucket_id)
        .await
        .map_err(ReportUploadError::DeleteOutdatedFailed)?;

    // Close the connection to prevent locking
    db_conn.close().await?;

    // Now, let's re-evaluate the capacity of that storage host
    HostCapacityTask::new(storage_provider.id.clone())
        .enqueue::<banyan_task::SqliteTaskStore>(&mut database)
        .await
        .map_err(ReportUploadError::UnableToEnqueueTask)?;

    let user_id = sqlx::query_scalar!("SELECT user_id FROM buckets WHERE id = $1", bucket_id)
        .fetch_one(&database)
        .await
        .map_err(ReportUploadError::QueryFailed)?;

    ReportStorageHostConsumptionTask::new(storage_provider.id.clone())
        .enqueue::<banyan_task::SqliteTaskStore>(&mut database)
        .await
        .map_err(ReportUploadError::UnableToEnqueueTask)?;

    ReportUserConsumptionTask::new(user_id)
        .enqueue::<banyan_task::SqliteTaskStore>(&mut database)
        .await
        .map_err(ReportUploadError::UnableToEnqueueTask)?;

    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

#[derive(Deserialize)]
pub struct ReportUploadRequest {
    data_size: i64,
    normalized_cids: Vec<String>,
    storage_authorization_id: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ReportUploadError {
    #[error("failed to run query against database: {0}")]
    QueryFailed(#[from] sqlx::Error),

    #[error("failed to mark the completed upload as current: {0}")]
    MarkCurrentFailed(sqlx::Error),

    #[error("failed to delete the metadatas no longer needed: {0}")]
    DeleteOutdatedFailed(sqlx::Error),

    #[error("failed to associate finalized uploaded with storage host: {0}")]
    NoUploadAssociation(sqlx::Error),

    #[error("failed to register storage grant as redeemed: {0}")]
    RedeemFailed(sqlx::Error),

    #[error("error occurred while recording a blocks present: {0}")]
    UnableToRecordBlock(sqlx::Error),

    #[error("could not enqueue task: {0}")]
    UnableToEnqueueTask(banyan_task::TaskStoreError),
}

impl IntoResponse for ReportUploadError {
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
    use uuid::Uuid;

    use crate::app::mock_app_state;
    use crate::database::models::{BlockLocations, MetadataState};
    use crate::database::test_helpers::{
        create_blocks, create_storage_grant, create_storage_hosts, data_generator, generate_cids,
        normalize_cids, redeem_storage_grant, sample_bucket, sample_metadata, sample_user,
        setup_database,
    };
    use crate::extractors::StorageProviderIdentity;
    use crate::hooks::storage::report_upload::{handler, ReportUploadRequest};

    #[tokio::test]
    async fn test_handler_for_staging() {
        let db = setup_database().await;
        let state = mock_app_state(db.clone());
        let mut conn = db.acquire().await.expect("connection");
        let staging_host_id = create_storage_hosts(&mut conn, "url1", "staging-service").await;
        let user_id = sample_user(&mut conn, "test@example.com").await;
        let bucket_id = sample_bucket(&mut conn, &user_id).await;
        let metadata_id = sample_metadata(&mut conn, &bucket_id, 1, MetadataState::Pending).await;
        let storage_grant_id =
            create_storage_grant(&mut conn, &staging_host_id, &user_id, 1_000_000).await;
        redeem_storage_grant(&mut conn, staging_host_id.as_str(), &storage_grant_id).await;
        let initial_cids: Vec<_> = normalize_cids(generate_cids(data_generator(0..2))).collect();
        create_blocks(&mut conn, initial_cids.iter().map(String::as_str)).await;

        let request = ReportUploadRequest {
            data_size: 1024,
            normalized_cids: initial_cids.clone(),
            storage_authorization_id: storage_grant_id.to_string(),
        };

        let result = handler(
            StorageProviderIdentity::default()
                .with_host_id(staging_host_id.as_str())
                .staging(),
            state,
            Path(Uuid::parse_str(&metadata_id).expect("valid uuid")),
            Json(request),
        )
        .await;

        assert!(result.is_ok());
        let block_locations = BlockLocations::get_all(&db).await.expect("block locations");
        assert_eq!(block_locations.len(), initial_cids.len());
    }
}
