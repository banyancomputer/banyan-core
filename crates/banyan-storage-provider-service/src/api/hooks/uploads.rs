use axum::extract::State;
use axum::headers::ContentLength;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Json, TypedHeader};
use jwt_simple::prelude::{Deserialize, Serialize};

use crate::app::AppState;
use crate::database::models::{AuthorizedStorage, Upload};
use crate::extractors::PlatformIdentity;

#[derive(Serialize, Deserialize)]
pub struct UploadRequest {
    pub(crate) metadata_id: String,
    pub(crate) client_id: String,
    pub(crate) grant_id: String,
    pub(crate) grant_size: i64,
}

pub async fn handler(
    State(state): State<AppState>,
    _: PlatformIdentity,
    TypedHeader(content_len): TypedHeader<ContentLength>,
    Json(request): Json<UploadRequest>,
) -> Result<Response, UploadRequestError> {
    let db = state.database();
    let reported_body_length = content_len.0;

    AuthorizedStorage::create_if_missing(
        &db,
        &request.client_id,
        &request.grant_id,
        request.grant_size,
    )
    .await?;

    let upload = Upload::get_by_metadata_id(&db, &request.metadata_id).await?;
    if let Some(upload) = upload {
        let msg = serde_json::json!({"upload_id": upload.id});
        return Ok((StatusCode::OK, Json(msg)).into_response());
    }

    let upload = start_upload(
        &db,
        &request.client_id,
        &request.metadata_id,
        reported_body_length,
    )
    .await?;

    let msg = serde_json::json!({"upload_id": upload.id});
    Ok((StatusCode::OK, Json(msg)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum UploadRequestError {
    #[error("database failure: {0}")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for UploadRequestError {
    fn into_response(self) -> Response {
        tracing::error!("{self}");
        let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
    }
}
