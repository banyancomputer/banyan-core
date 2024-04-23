use std::collections::HashSet;

use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use banyan_task::TaskLikeExt;

use crate::app::AppState;
use crate::extractors::PlatformIdentity;
use crate::tasks::PruneBlocksTask;
use crate::utils::is_valid_cid;

pub async fn handler(
    _ci: PlatformIdentity,
    State(state): State<AppState>,
    Json(cids_to_prune): Json<HashSet<String>>,
) -> Result<Response, PruneBlocksError> {
    let cids_to_prune = cids_to_prune.into_iter().collect::<Vec<_>>();
    if cids_to_prune.iter().any(|c| !is_valid_cid(c)) {
        return Err(PruneBlocksError::InvalidCid);
    }

    let db = state.database();
    let mut conn = db.acquire().await?;

    PruneBlocksTask::new(cids_to_prune)
        .enqueue::<banyan_task::SqliteTaskStore>(&mut conn)
        .await?;
    Ok((StatusCode::OK, ()).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum PruneBlocksError {
    #[error("database failure: {0}")]
    Database(#[from] sqlx::Error),

    #[error("request contained an invalid CID")]
    InvalidCid,

    #[error("could not enqueue task: {0}")]
    UnableToEnqueueTask(#[from] banyan_task::TaskStoreError),
}

impl IntoResponse for PruneBlocksError {
    fn into_response(self) -> Response {
        tracing::error!("{self}");
        let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
    }
}
