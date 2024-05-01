use std::collections::HashSet;

use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;
use banyan_task::{SqliteTaskStore, TaskLikeExt};
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::app::AppState;
use crate::database::models::{Blocks, Uploads};
use crate::extractors::PlatformIdentity;
use crate::tasks::RedistributeDataTask;

#[derive(Serialize, Deserialize)]
pub struct DistributeData {
    metadata_id: String,
    storage_grant_id: String,
    storage_grant_size: i64,
    block_cids: Vec<String>,
    new_host_id: String,
    new_host_url: String,
}

pub async fn handler(
    _: PlatformIdentity,
    State(state): State<AppState>,
    Json(distribute_data): Json<DistributeData>,
) -> Result<Response, DistributeBlocksError> {
    let mut conn = state.database().acquire().await?;
    let metadata_id = &distribute_data.metadata_id;

    let task = RedistributeDataTask {
        metadata_id: distribute_data.metadata_id.clone(),
        storage_grant_id: distribute_data.storage_grant_id.clone(),
        storage_grant_size: distribute_data.storage_grant_size,
        block_cids: distribute_data.block_cids.clone(),
        new_host_id: distribute_data.new_host_id.clone(),
        new_host_url: distribute_data.new_host_url.clone(),
    };
    if SqliteTaskStore::is_present(&mut conn, &task).await? {
        return Ok((StatusCode::OK, ()).into_response());
    }

    Uploads::get_by_metadata_id(&mut conn, metadata_id).await?;
    let blocks: Vec<Blocks> =
        Blocks::get_blocks_by_cid(&mut conn, &distribute_data.block_cids).await?;

    if blocks.len() != distribute_data.block_cids.len() {
        let block_cids: HashSet<String> = distribute_data.block_cids.into_iter().collect();
        let blocks_cids_set: HashSet<String> =
            blocks.iter().map(|block| block.cid.clone()).collect();

        let missing_cids = block_cids
            .symmetric_difference(&blocks_cids_set)
            .collect::<HashSet<_>>();

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
            DistributeBlocksError::BadRequest(_) | DistributeBlocksError::Database(_) => {
                let err_msg = serde_json::json!({ "msg": "invalid request" });
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
        }
    }
}
