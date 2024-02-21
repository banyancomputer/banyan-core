use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use banyan_object_store::{ObjectStore, ObjectStoreError, ObjectStorePath};
use banyan_task::{SqliteTaskStore, TaskStoreError};
use serde::Deserialize;

use crate::app::AppState;
use crate::database::models::{Blocks, Uploads};
use crate::extractors::PlatformIdentity;
use crate::tasks::UploadBlocksTask;

#[derive(Deserialize)]
pub struct DeleteBlocksRequest {
    pub normalized_cids: Vec<String>,
    pub metadata_id: String,
}

pub async fn handler(
    _: PlatformIdentity,
    State(state): State<AppState>,
    Json(request): Json<DeleteBlocksRequest>,
) -> Result<Response, BlocksDeleteError> {
    let mut transaction = state.database().begin().await?;
    let metadata_id = request.metadata_id;
    let task = UploadBlocksTask::new_with_metadata_id(metadata_id.to_string());
    if !SqliteTaskStore::is_present(&mut transaction, &task).await? {
        // there wasn't a previously scheduled upload blocks task to distribute
        // the data to a new storage provider we should not delete the blocks
        return Ok((StatusCode::BAD_REQUEST, ()).into_response());
    }
    Uploads::delete_by_metadata_id(&mut transaction, &metadata_id).await?;
    let deleted_blocks =
        Blocks::delete_blocks_by_cid(&mut transaction, &request.normalized_cids).await?;

    if deleted_blocks.rows_affected() != request.normalized_cids.len() as u64 {
        return Err(BlocksDeleteError::DeleteFailed(format!(
            "deleted {} vs cids {} for metadata {}",
            deleted_blocks.rows_affected(),
            request.normalized_cids.len(),
            metadata_id,
        )));
    }

    let store = ObjectStore::new(state.upload_store_connection())?;
    for block_cid in request.normalized_cids.iter() {
        let location = ObjectStorePath::from(format!("{}/{}.bin", &metadata_id, block_cid));
        store.delete(&location).await?;
    }
    transaction.commit().await?;

    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum BlocksDeleteError {
    #[error("delete failed: {0}")]
    DeleteFailed(String),
    #[error("database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("delete block error: {0}")]
    DeleteBlockError(#[from] ObjectStoreError),
    #[error("task store error: {0}")]
    TaskStoreError(#[from] TaskStoreError),
}

impl IntoResponse for BlocksDeleteError {
    fn into_response(self) -> Response {
        use BlocksDeleteError::*;
        match self {
            DatabaseError(_) | DeleteBlockError(_) | TaskStoreError(_) => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            DeleteFailed(_) => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({ "msg": "could not delete blocks" });
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use banyan_task::TaskStore;

    use super::*;
    use crate::api::admin::delete_blocks::BlocksDeleteError;
    use crate::app::mock_app_state;
    use crate::database::{test_helpers, Database};

    pub async fn get_block_cids(db: &Database, block_ids: Vec<String>) -> Vec<String> {
        let mut cids = Vec::new();
        for block_id in block_ids {
            let block = sqlx::query!("SELECT cid FROM blocks WHERE id = $1", block_id)
                .fetch_one(db)
                .await
                .expect("block cids");
            cids.push(block.cid);
        }
        cids
    }
    async fn setup_test_environment() -> (Database, String, String, Vec<String>) {
        let db = test_helpers::setup_database().await;
        let metadata_id = "test_metadata_id";
        let client_id = test_helpers::create_client(
            &db,
            "test_platform",
            "test_fingerprint",
            "test_public_key",
        )
        .await;
        let upload_id = test_helpers::create_upload(&db, &client_id, metadata_id, 1000).await;
        let block_ids = test_helpers::sample_blocks(&db, 4, &upload_id).await;
        (db, metadata_id.to_string(), client_id, block_ids)
    }

    #[tokio::test]
    async fn test_handle_short_circuits_on_missing_task() {
        let (db, metadata_id, _, block_ids) = setup_test_environment().await;

        let result = handler(
            PlatformIdentity {},
            mock_app_state(db.clone()),
            Json(DeleteBlocksRequest {
                normalized_cids: block_ids,
                metadata_id: metadata_id,
            }),
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap().into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_handler_returns_success() {
        let (mut db, metadata_id, _, block_ids) = setup_test_environment().await;
        SqliteTaskStore::enqueue(
            &mut db,
            UploadBlocksTask::new_with_metadata_id(metadata_id.clone()),
        )
        .await
        .unwrap();
        let blocks_cids = get_block_cids(&db, block_ids.clone()).await;
        let app_state = mock_app_state(db.clone());
        test_helpers::save_blocks_to_storage(
            &app_state.upload_store_connection(),
            &metadata_id,
            blocks_cids.clone(),
        )
        .await;

        let result = handler(
            PlatformIdentity {},
            app_state,
            Json(DeleteBlocksRequest {
                normalized_cids: blocks_cids,
                metadata_id: metadata_id,
            }),
        )
        .await;

        assert!(result.is_ok());
        let response = result.unwrap().into_response();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_rollback_on_error() {
        let (mut db, metadata_id, _, block_ids) = setup_test_environment().await;
        SqliteTaskStore::enqueue(
            &mut db,
            UploadBlocksTask::new_with_metadata_id(metadata_id.clone()),
        )
        .await
        .unwrap();

        let result = handler(
            PlatformIdentity {},
            mock_app_state(db.clone()),
            Json(DeleteBlocksRequest {
                normalized_cids: block_ids,
                metadata_id: metadata_id,
            }),
        )
        .await;

        assert!(result.is_err());
        assert!(matches!(result, Err(BlocksDeleteError::DeleteFailed(_))));
    }
}
