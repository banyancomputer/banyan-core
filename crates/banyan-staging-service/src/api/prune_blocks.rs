use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use banyan_task::{SqliteTaskStore, TaskLikeExt};
use cid::Cid;

use crate::app::AppState;
use crate::extractors::PlatformIdentity;
use crate::tasks::PruneBlocksTask;
use crate::utils::NORMALIZED_CID_BASE;

pub async fn handler(
    _ci: PlatformIdentity,
    State(state): State<AppState>,
    Json(prune_cids): Json<Vec<Cid>>,
) -> Result<Response, PruneBlocksError> {
    // Normalize the block CIDs, warn but keep going on any invalid ones
    let mut prune_block_list = Vec::new();
    for cid in prune_cids.into_iter() {
        match cid.to_string_of_base(NORMALIZED_CID_BASE) {
            Ok(cid_str) => prune_block_list.push(cid_str),
            Err(err) => {
                tracing::warn!("failed to normalize CID from platform prune request: {err}")
            }
        }
    }

    let mut conn = state.database().acquire().await?;
    PruneBlocksTask::new(prune_block_list)
        .enqueue::<SqliteTaskStore>(&mut conn)
        .await?;
    Ok((StatusCode::OK, ()).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum PruneBlocksError {
    #[error("could not enqueue task: {0}")]
    UnableToEnqueueTask(#[from] banyan_task::TaskStoreError),
    #[error("database: {0}")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for PruneBlocksError {
    fn into_response(self) -> Response {
        tracing::error!("{self}");
        let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
    }
}
