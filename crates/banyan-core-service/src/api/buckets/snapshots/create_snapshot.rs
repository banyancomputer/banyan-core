use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::app::AppState;
use crate::extractors::ApiIdentity;

pub async fn handler(
    api_id: ApiIdentity,
    State(state): State<AppState>,
    Path((bucket_id, metadata_id)): Path<(Uuid, Uuid)>,
) -> Result<Response, CreateSnapshotError> {
    let database = state.database();

    let bucket_id = bucket_id.to_string();
    let metadata_id = metadata_id.to_string();

    todo!()
}

#[derive(Debug, thiserror::Error)]
pub enum CreateSnapshotError {
    #[error("no matching metadata for the current account")]
    NotFound,
}

impl IntoResponse for CreateSnapshotError {
    fn into_response(self) -> Response {
        match &self {
            CreateSnapshotError::NotFound => {
                let err_msg = serde_json::json!({"msg": "not found"});
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            }
            _ => {
                tracing::error!("encountered error creating snapshot: {self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
