use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;
use banyan_task::{SqliteTaskStore, TaskLikeExt};
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::app::AppState;
use crate::database::models::Uploads;
use crate::extractors::PlatformIdentity;
use crate::tasks::ReplicateDataTask;

#[derive(Serialize, Deserialize)]
pub struct ReplicateData {
    pub metadata_id: String,
    pub block_cids: Vec<String>,
    pub new_host_id: String,
    pub new_storage_grant_id: String,
    pub new_storage_grant_size: i64,
    pub new_host_url: String,
    pub old_host_id: String,
    pub old_host_url: String,
}

pub async fn handler(
    _: PlatformIdentity,
    State(state): State<AppState>,
    Json(replicate_data): Json<ReplicateData>,
) -> Result<Response, ReplicateError> {
    let db = state.database();
    let mut conn = db.acquire().await?;
    let metadata_id = &replicate_data.metadata_id;

    let task = ReplicateDataTask {
        metadata_id: replicate_data.metadata_id.clone(),
        block_cids: replicate_data.block_cids.clone(),
        new_host_id: replicate_data.new_host_id.clone(),
        new_host_url: replicate_data.new_host_url.clone(),
        new_storage_grant_id: replicate_data.new_storage_grant_id.clone(),
        new_storage_grant_size: replicate_data.new_storage_grant_size,
        old_host_id: replicate_data.old_host_id.clone(),
        old_host_url: replicate_data.old_host_url.clone(),
    };
    if SqliteTaskStore::is_present(&mut conn, &task).await? {
        return Ok((StatusCode::OK, ()).into_response());
    }

    Uploads::get_by_metadata_id(&db, metadata_id).await?;

    // TODO: early validate that the blocks do exist on the old host
    task.enqueue::<SqliteTaskStore>(&mut conn).await?;

    Ok((StatusCode::OK, ()).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum ReplicateError {
    #[error("a database error occurred during an upload {0}")]
    Database(#[from] sqlx::Error),
    #[error("could not enqueue task: {0}")]
    UnableToEnqueueTask(#[from] banyan_task::TaskStoreError),
}

impl IntoResponse for ReplicateError {
    fn into_response(self) -> Response {
        match self {
            ReplicateError::UnableToEnqueueTask(_) => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            ReplicateError::Database(_) => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({ "msg": "upload not found" });
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
        }
    }
}
