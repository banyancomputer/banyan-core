use axum::extract::State;
use axum::headers::ContentLength;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Json, TypedHeader};
use jwt_simple::prelude::{Deserialize, Serialize};

use crate::api::upload::{start_upload, UploadError};
use crate::app::AppState;
use crate::database::models::AuthorizedStorage;
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
) -> Result<Response, UploadError> {
    let db = state.database();
    let reported_body_length = content_len.0;
    AuthorizedStorage::create_if_missing(
        &db,
        &request.client_id,
        &request.grant_id,
        request.grant_size,
    )
    .await?;
    // Start the upload with these specifications
    let upload = start_upload(
        &db,
        &request.client_id,
        &request.metadata_id,
        reported_body_length,
    )
    .await?;

    // Respond with the upload id
    let msg = serde_json::json!({"upload_id": upload.id});
    Ok((StatusCode::OK, Json(msg)).into_response())
}
