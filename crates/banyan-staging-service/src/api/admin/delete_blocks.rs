use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use banyan_object_store::{ObjectStore, ObjectStoreError, ObjectStorePath};
use serde::Deserialize;

use crate::app::AppState;
use crate::database::models::{Blocks, Uploads};
use crate::extractors::PlatformIdentity;

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
}

impl IntoResponse for BlocksDeleteError {
    fn into_response(self) -> Response {
        use BlocksDeleteError::*;
        match self {
            DatabaseError(_) | DeleteBlockError(_) => {
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
