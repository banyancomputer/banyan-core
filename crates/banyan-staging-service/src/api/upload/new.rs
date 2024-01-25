use axum::extract::{BodyStream, State};
use axum::headers::{ContentLength, ContentType};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Json, TypedHeader};
use banyan_car_analyzer::{CarReport, StreamingCarAnalyzer, StreamingCarAnalyzerError};
use banyan_object_store::ObjectStore;
use banyan_task::TaskLikeExt;
use futures::{TryStream, TryStreamExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::db::{
    complete_upload, fail_upload, get_upload, start_upload, write_block_to_tables, Upload,
};
use super::error::UploadError;
use crate::app::AppState;
use crate::database::Database;
use crate::extractors::AuthenticatedClient;
use crate::tasks::ReportUploadTask;

#[derive(Deserialize, Serialize)]
pub struct BlockUploadRequest {
    metadata_id: Uuid,
    content_hash: String,

    // Optional additional details about the nature of the upload
    #[serde(flatten)]
    details: BlockUploadDetails,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum BlockUploadDetails {
    Ongoing { completed: bool, upload_id: String },
    OneOff,
}

pub async fn handler(
    State(state): State<AppState>,
    client: AuthenticatedClient,
    store: ObjectStore,
    TypedHeader(content_len): TypedHeader<ContentLength>,
    Json(request): Json<BlockUploadRequest>,
) -> Result<Response, UploadError> {
    let mut db = state.database();
    let reported_body_length = content_len.0;
    if reported_body_length > client.remaining_storage() {
        return Err(UploadError::InsufficientAuthorizedStorage(
            reported_body_length,
            client.remaining_storage(),
        ));
    }
    match request.details {
        BlockUploadDetails::OneOff => {
            // Create a new upload for the one off
            let upload = start_upload(
                &db,
                &client.id(),
                &request.metadata_id,
                reported_body_length,
            )
            .await?;

            // TODO Handle data

            // Complete the upload
            complete_upload(
                &db,
                reported_body_length as i64,
                &request.content_hash,
                &upload.id,
            )
            .await?;
        }
        BlockUploadDetails::Ongoing {
            completed,
            upload_id,
        } => {
            // Assume that the upload has already been created via the `new` endpoint
            let upload = get_upload(&db, client.id(), request.metadata_id).await?;
        }
    }
    let msg = serde_json::json!({"status": "ok"});
    Ok((StatusCode::OK, Json(msg)).into_response())
}
