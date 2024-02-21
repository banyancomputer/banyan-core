use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;
use banyan_task::{SqliteTaskStore, TaskLikeExt};
use http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::api::admin::distribute::DistributeError::{Database, UnableToEnqueueTask};
use crate::app::AppState;
use crate::database::models::Uploads;
use crate::extractors::PlatformIdentity;
use crate::tasks::RedistributeDataTask;

#[derive(Serialize, Deserialize)]
pub struct DistributeData {
    metadata_id: String,
    grant_id: String,
    new_host_id: String,
    new_host_url: String,
}

pub async fn handler(
    _: PlatformIdentity,
    State(state): State<AppState>,
    Json(distribute_data): Json<DistributeData>,
) -> Result<Response, DistributeError> {
    let db = state.database();
    let metadata_id = &distribute_data.metadata_id;

    Uploads::get_by_metadata_id(&db, metadata_id).await?;

    let task = RedistributeDataTask {
        metadata_id: distribute_data.metadata_id.clone(),
        grant_id: distribute_data.grant_id.clone(),
        new_host_id: distribute_data.new_host_id.clone(),
        new_host_url: distribute_data.new_host_url.clone(),
    };
    let mut transaction = db.begin().await?;
    if SqliteTaskStore::is_present(&mut transaction, &task).await? {
        return Ok((StatusCode::OK, ()).into_response());
    }
    task.enqueue_with_connection::<SqliteTaskStore>(&mut transaction)
        .await?;
    transaction.commit().await?;

    Ok((StatusCode::OK, ()).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum DistributeError {
    #[error("a database error occurred during an upload {0}")]
    Database(#[from] sqlx::Error),
    #[error("could not enqueue task: {0}")]
    UnableToEnqueueTask(#[from] banyan_task::TaskStoreError),
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
