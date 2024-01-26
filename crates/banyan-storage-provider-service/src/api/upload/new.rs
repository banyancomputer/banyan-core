use axum::{
    extract::State,
    headers::ContentLength,
    response::{IntoResponse, Response},
    Json, TypedHeader,
};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{app::AppState, extractors::AuthenticatedClient};

use super::{
    db::{start_upload, Upload},
    error::UploadError,
};

// Requests need only the associated metadata id
#[derive(Serialize, Deserialize)]
pub struct NewUploadRequest {
    metadata_id: Uuid,
}

pub async fn handler(
    State(state): State<AppState>,
    client: AuthenticatedClient,
    TypedHeader(content_len): TypedHeader<ContentLength>,
    Json(request): Json<NewUploadRequest>,
) -> Result<Response, UploadError> {
    let db = state.database();
    let reported_body_length = content_len.0;
    if reported_body_length > client.remaining_storage() {
        return Err(UploadError::InsufficientAuthorizedStorage(
            reported_body_length,
            client.remaining_storage(),
        ));
    }

    // Start the upload with these specifications
    let upload = start_upload(
        &db,
        &client.id(),
        &request.metadata_id,
        reported_body_length,
    )
    .await?;

    // Respond with the upload id
    let msg = serde_json::json!({"upload_id": upload.id});
    Ok((StatusCode::OK, Json(msg)).into_response())
}
