use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use banyan_task::{SqliteTaskStore, TaskStore};

use crate::app::AppState;

pub async fn handler(state: State<AppState>) -> Result<Response, ReadMetricsError> {
    let database = state.database();
    let task_store = SqliteTaskStore::new(database);
    let metrics = task_store.metrics().await.map_err(ReadMetricsError::UnableToReadMetrics)?;
    Ok((StatusCode::OK, Json(metrics)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum ReadMetricsError {
    #[error("could not read metrics")]
    UnableToReadMetrics(banyan_task::TaskStoreError),
}

impl IntoResponse for ReadMetricsError {
    fn into_response(self) -> Response {
        {
            tracing::error!("encountered error reading metrics: {self}");
            let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
        }
    }
}