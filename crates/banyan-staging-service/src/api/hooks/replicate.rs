use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;
use banyan_task::{SqliteTaskStore, TaskLikeExt};
use http::StatusCode;

use crate::app::AppState;
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
