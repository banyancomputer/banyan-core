use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::api::upload::Upload;
use crate::app::AppState;
use crate::extractors::PlatformIdentity;

pub async fn handler(
    _: PlatformIdentity,
    State(state): State<AppState>,
    Path(metadata_id): Path<String>,
) -> Result<Response, GetUploadError> {
    let db = state.database();
    let upload = match Upload::get_by_metadata_id(&db, &metadata_id).await? {
        Some(upload) => upload,
        None => return Err(GetUploadError::UploadNotFound),
    };

    let msg = serde_json::json!({
        "id": upload.id,
        "client_id": upload.client_id,
        "metadata_id": upload.metadata_id,
        "base_path": upload.base_path,
        "reported_size": upload.reported_size,
        "state": upload.state,
    });
    Ok((StatusCode::OK, Json(msg)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum GetUploadError {
    #[error("failed to lookup upload: {0}")]
    LookupFailed(#[from] sqlx::Error),

    #[error("upload not found")]
    UploadNotFound,
}

impl IntoResponse for GetUploadError {
    fn into_response(self) -> Response {
        match self {
            GetUploadError::UploadNotFound => {
                let err_msg = serde_json::json!({"msg": "upload not found"});
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            }
            _ => {
                let err_msg = serde_json::json!({"msg": "a backend service issue occurred"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
