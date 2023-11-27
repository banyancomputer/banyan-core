use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use banyan_task::TaskLikeExt;

use crate::app::AppState;
use crate::extractors::PlatformIdentity;
use crate::tasks::{PruneBlock, PruneBlocksTask};

pub async fn handler(
    _ci: PlatformIdentity,
    State(state): State<AppState>,
    Json(prune_blocks): Json<Vec<PruneBlock>>,
) -> Result<Response, PruneBlocksError> {
    let mut db = state.database();
    PruneBlocksTask::new(prune_blocks)
        .enqueue::<banyan_task::SqliteTaskStore>(&mut db)
        .await
        .map_err(PruneBlocksError::UnableToEnqueueTask)?;
    Ok((StatusCode::OK, ()).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum PruneBlocksError {
    #[error("could not enqueue task: {0}")]
    UnableToEnqueueTask(banyan_task::TaskStoreError),
}

impl IntoResponse for PruneBlocksError {
    fn into_response(self) -> Response {
        {
            tracing::error!("{self}");
            let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
        }
    }
}
