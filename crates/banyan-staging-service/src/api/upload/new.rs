use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::error::UploadError;
use crate::app::AppState;
use crate::database::models::CreateUpload;
use crate::extractors::AuthenticatedClient;

#[derive(Serialize, Deserialize)]
pub struct NewUploadRequest {
    metadata_id: Uuid,
    session_data_size: u64,
}

pub async fn handler(
    client: AuthenticatedClient,
    State(state): State<AppState>,
    Json(request): Json<NewUploadRequest>,
) -> Result<Response, UploadError> {
    let db = state.database();
    let mut conn = db.acquire().await?;

    let client_id_str = client.id().to_string();
    let metadata_id_str = request.metadata_id.to_string();

    let upload_id = CreateUpload {
        client_id: &client_id_str,
        metadata_id: &metadata_id_str,
        reported_size: request.session_data_size as i64,
    }
    .save(&mut conn)
    .await?;

    tracing::error!(client_id = ?client_id_str, metadata_id = ?metadata_id_str, upload_id = ?upload_id, "created upload session");

    let msg = serde_json::json!({"upload_id": upload_id});
    Ok((StatusCode::OK, Json(msg)).into_response())
}
