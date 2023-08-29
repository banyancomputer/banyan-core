use axum::extract::{BodyStream, Json};
use axum::headers::ContentType;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::TypedHeader;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::extractors::{AuthenticatedClient, Database, UploadStore};

/// Limit on the size of the JSON request that accompanies an upload.
const UPLOAD_REQUEST_SIZE_LIMIT: u64 = 100 * 1_024;

#[derive(Deserialize, Serialize)]
pub struct UploadRequest {
    metadata_id: Uuid,
}

pub async fn handler(
    _db: Database,
    client: AuthenticatedClient,
    _store: UploadStore,
    TypedHeader(content_type): TypedHeader<ContentType>,
    body: BodyStream,
) -> Result<Response, UploadError> {
    let mime_ct = mime::Mime::from(content_type);
    let boundary = multer::parse_boundary(mime_ct).unwrap();
    let constraints = multer::Constraints::new()
        .allowed_fields(vec!["request-data", "car-upload"])
        .size_limit(
            multer::SizeLimit::new()
                .for_field("request-data", UPLOAD_REQUEST_SIZE_LIMIT)
                .for_field("car-upload", client.remaining_storage() as u64),
        );

    let mut multipart = multer::Multipart::with_constraints(body, boundary, constraints);

    // Process the request data
    let request_data = multipart
        .next_field()
        .await
        .map_err(UploadError::RequestFieldUnavailable)?
        .ok_or(UploadError::RequestFieldMissing)?;

    // TODO: validate name is request-data (request_data_field.name())
    // TODO: validate type is application/json (request_data_field.content_type())

    let _request: UploadRequest = request_data
        .json()
        .await
        .map_err(UploadError::InvalidRequestData)?;

    // check content length against quota
    // record that an upload is in progress
    // collect upload in a temporary directory
    // during upload, if it goes over content length warn and start watching remaining authorized
    // storage
    // if upload errors clean up files and record the failure in the database with the uploaded amount
    // if upload succeeds queue task to report back to platform

    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum UploadError {
    #[error("request's data payload was malformed")]
    InvalidRequestData(multer::Error),

    #[error("failed to acquire request field from body")]
    RequestFieldUnavailable(multer::Error),

    #[error("we expected a request field but received nothing")]
    RequestFieldMissing,
}

impl IntoResponse for UploadError {
    fn into_response(self) -> Response {
        use UploadError::*;

        match &self {
            InvalidRequestData(_) | RequestFieldUnavailable(_) | RequestFieldMissing => {
                let err_msg = serde_json::json!({ "msg": format!("{self}") });
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
        }
    }
}
