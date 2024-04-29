use std::collections::HashSet;

use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;
use banyan_task::{SqliteTaskStore, TaskLikeExt};
use http::StatusCode;

use crate::app::AppState;
use crate::clients::{
    CoreServiceClient, CoreServiceError, StorageProviderClient, StorageProviderError,
};
use crate::database::models::Uploads;
use crate::extractors::PlatformIdentity;
use crate::tasks::ReplicateDataTask;

pub async fn handler(
    _: PlatformIdentity,
    State(state): State<AppState>,
    Json(task): Json<ReplicateDataTask>,
) -> Result<Response, ReplicateError> {
    let db = state.database();
    let mut conn = db.acquire().await?;

    if SqliteTaskStore::is_present(&mut conn, &task).await? {
        return Ok((StatusCode::OK, ()).into_response());
    }

    Uploads::get_by_metadata_id(&db, &task.metadata_id).await?;
    let core_client = CoreServiceClient::new(
        state.secrets().service_signing_key(),
        state.service_name(),
        state.platform_name(),
        state.platform_hostname(),
    )?;
    let provider_credentials = core_client
        .request_provider_token(&task.old_host_id)
        .await?;
    let new_client = StorageProviderClient::new(&task.old_host_url, &provider_credentials.token)?;
    let res = new_client
        .blocks_present(task.block_cids.as_slice())
        .await?;
    let res_set: HashSet<_> = res.into_iter().collect();
    let missing_blocks: Vec<_> = task
        .block_cids
        .iter()
        .filter(|i| !res_set.contains(*i))
        .collect();
    if !missing_blocks.is_empty() {
        return Ok((StatusCode::BAD_REQUEST, Json(missing_blocks)).into_response());
    }

    task.enqueue::<SqliteTaskStore>(&mut conn).await?;

    Ok((StatusCode::OK, ()).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum ReplicateError {
    #[error("a database error occurred during an upload {0}")]
    Database(#[from] sqlx::Error),
    #[error("could not enqueue task: {0}")]
    UnableToEnqueueTask(#[from] banyan_task::TaskStoreError),
    #[error("core service error: {0}")]
    CoreServiceError(#[from] CoreServiceError),
    #[error("staging service error: {0}")]
    StagingServiceError(#[from] StorageProviderError),
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
            ReplicateError::CoreServiceError(e) => {
                let err_msg = serde_json::json!({ "msg": format!("could not connect to core service error {e}") });
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            ReplicateError::StagingServiceError(e) => {
                let err_msg = serde_json::json!({ "msg": format!("could not connect to storage provider error {e}") });
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
        }
    }
}
