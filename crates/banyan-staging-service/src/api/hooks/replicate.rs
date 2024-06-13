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
use crate::extractors::PlatformIdentity;
use crate::tasks::ReplicateDataTask;

pub async fn handler(
    _: PlatformIdentity,
    State(state): State<AppState>,
    Json(task): Json<ReplicateDataTask>,
) -> Result<Response, ReplicateError> {
    let mut conn = state.database().acquire().await?;

    if SqliteTaskStore::is_present(&mut conn, &task).await? {
        return Err(ReplicateError::AlreadyProcessed);
    }

    let core_client = CoreServiceClient::new(
        state.secrets().service_signing_key(),
        state.service_name(),
        state.platform_name(),
        state.platform_hostname(),
    )?;
    let provider_credentials = core_client
        .request_provider_token(&task.old_host_id)
        .await?;
    let old_host = StorageProviderClient::new(&task.old_host_url, &provider_credentials.token)?;
    let res = old_host.blocks_present(task.block_cids.as_slice()).await?;
    let res_set: HashSet<_> = res.into_iter().collect();
    let missing_blocks: Vec<_> = task
        .block_cids
        .iter()
        .filter(|i| !res_set.contains(*i))
        .map(|s| s.as_str())
        .collect();
    if !missing_blocks.is_empty() {
        return Err(ReplicateError::MissingBlocksError(
            missing_blocks.join(", "),
        ));
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
    #[error("missing blocks on old host: {0}")]
    MissingBlocksError(String),
    #[error("already scheduled")]
    AlreadyProcessed,
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
            ReplicateError::StagingServiceError(_)
            | ReplicateError::CoreServiceError(_)
            | ReplicateError::AlreadyProcessed
            | ReplicateError::MissingBlocksError(_) => {
                let err_msg = serde_json::json!({ "msg": format!("{self}") });
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
        }
    }
}
