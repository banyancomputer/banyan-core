use std::collections::HashSet;

use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;
use banyan_task::{SqliteTaskStore, TaskLikeExt};
use http::StatusCode;

use crate::app::AppState;
use crate::database::models::{Blocks, Uploads};
use crate::extractors::PlatformIdentity;
use crate::tasks::RedistributeDataTask;

pub async fn handler(
    _: PlatformIdentity,
    State(state): State<AppState>,
    Json(task): Json<RedistributeDataTask>,
) -> Result<Response, DistributeBlocksError> {
    let mut conn = state.database().acquire().await?;
    let metadata_id = &task.metadata_id;

    if SqliteTaskStore::is_present(&mut conn, &task).await? {
        return Err(DistributeBlocksError::AlreadyProcessed);
    }

    Uploads::get_by_metadata_id(&mut conn, metadata_id).await?;
    let blocks: Vec<Blocks> = Blocks::get_blocks_by_cid(&mut conn, &task.block_cids).await?;
    let blocks_cids_set: HashSet<String> = blocks.into_iter().map(|block| block.cid).collect();
    let missing_cids: Vec<_> = task
        .block_cids
        .iter()
        .filter(|i| !blocks_cids_set.contains(*i))
        .map(|s| s.as_str())
        .collect();
    if !missing_cids.is_empty() {
        return Err(DistributeBlocksError::BadRequest(format!(
            "block CIDs do not match: {:?}",
            missing_cids
        )));
    }

    task.enqueue::<SqliteTaskStore>(&mut conn).await?;

    Ok((StatusCode::OK, ()).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum DistributeBlocksError {
    #[error("a database error occurred: {0}")]
    Database(#[from] sqlx::Error),
    #[error("already scheduled")]
    AlreadyProcessed,
    #[error("could not task: {0}")]
    BadRequest(String),
    #[error("could not enqueue task: {0}")]
    UnableToEnqueueTask(#[from] banyan_task::TaskStoreError),
}

impl IntoResponse for DistributeBlocksError {
    fn into_response(self) -> Response {
        match self {
            DistributeBlocksError::UnableToEnqueueTask(_) => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            DistributeBlocksError::AlreadyProcessed
            | DistributeBlocksError::BadRequest(_)
            | DistributeBlocksError::Database(_) => {
                let err_msg = serde_json::json!({ "msg": "invalid request" });
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
        }
    }
}
