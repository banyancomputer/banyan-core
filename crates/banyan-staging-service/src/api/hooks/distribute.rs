use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;
use banyan_task::{SqliteTaskStore, TaskLike, TaskLikeExt};
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::api::hooks::distribute::DistributeError::{Database, UnableToEnqueueTask};
use crate::app::AppState;
use crate::database::models::Uploads;
use crate::database::DatabaseError;
use crate::extractors::{AuthenticatedClient, PlatformIdentity};
use crate::tasks::RedistributeDataTask;

#[derive(Serialize, Deserialize)]
pub struct DistributeData {
    metadata_id: String,
    new_host_id: String,
    new_host_url: String,
}

pub async fn handler(
    _: PlatformIdentity,
    State(state): State<AppState>,
    Json(distribute_data): Json<DistributeData>,
) -> Result<Response, DistributeError> {
    let mut db = state.database();
    let metadata_id = &distribute_data.metadata_id;

    Uploads::get_by_metadata_id(&db, metadata_id).await?;
    let task = RedistributeDataTask::new(
        distribute_data.metadata_id.clone(),
        distribute_data.new_host_id.clone(),
        distribute_data.new_host_url.clone(),
    );
    let unique_key = match task.unique_key() {
        Some(unique_key) => unique_key,
        None => {
            return Ok((StatusCode::BAD_REQUEST, ()).into_response());
        }
    };

    let mut transaction = db.begin().await?;
    if SqliteTaskStore::is_key_present(
        &mut transaction,
        unique_key.as_str(),
        &RedistributeDataTask::TASK_NAME,
    )
    .await
    .map_err(UnableToEnqueueTask)?
    {
        return Ok((StatusCode::OK, ()).into_response());
    }
    transaction.commit().await?;

    task.enqueue::<SqliteTaskStore>(&mut db)
        .await
        .map_err(UnableToEnqueueTask)?;

    Ok((StatusCode::OK, ()).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum DistributeError {
    #[error("a database error occurred during an upload {0}")]
    Database(#[from] sqlx::Error),
    #[error("could not enqueue task: {0}")]
    UnableToEnqueueTask(banyan_task::TaskStoreError),
}

impl IntoResponse for DistributeError {
    fn into_response(self) -> Response {
        match self {
            UnableToEnqueueTask(_) => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            Database(_) => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({ "msg": "upload not found" });
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
        }
    }
}
