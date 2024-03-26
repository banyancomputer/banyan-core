use axum::response::{IntoResponse, Response};
use axum::Json;
use banyan_car_analyzer::StreamingCarAnalyzerError;
use http::StatusCode;

use crate::database::{map_sqlx_error, DatabaseError};

#[derive(Debug, thiserror::Error)]
pub enum UploadError {
    #[error("a database error occurred during an upload {0}")]
    Database(#[from] DatabaseError),

    #[error("we expected a data field but received nothing")]
    DataFieldMissing,

    #[error("failed to acquire data field from body")]
    DataFieldUnavailable(multer::Error),

    #[error("failed to enqueue a task: {0}")]
    FailedToEnqueueTask(#[from] banyan_task::TaskStoreError),

    #[error("unauthorized write, metadata and upload ids are not congruent")]
    IdMismatch,

    #[error("account is not authorized to store {0} bytes, {1} bytes are still authorized")]
    InsufficientAuthorizedStorage(u64, u64),

    #[error("request contained invalid CID")]
    InvalidCid,

    #[error("cannot write blocks to a CAR file directly")]
    CarFile,

    #[error("request's data payload was malformed")]
    InvalidRequestData(multer::Error),

    #[error("failed to acquire request field from body")]
    RequestFieldUnavailable(multer::Error),

    #[error("we expected a request field but received nothing")]
    RequestFieldMissing,

    #[error("uploaded file was not a properly formatted car file")]
    ParseError(#[from] StreamingCarAnalyzerError),

    #[error("Data in request mismatched attached CID")]
    MismatchedCid((String, String)),

    #[error("failed to read from client")]
    ReadFailed(multer::Error),

    #[error("failed to write to storage backend")]
    ObjectStore(banyan_object_store::ObjectStoreError),

    #[error("tried to write to a completed upload")]
    UploadIsComplete,

    #[error("failed to locate upload")]
    UploadLookupFailure,

    #[error("not yet supported")]
    NotSupported,
}

impl IntoResponse for UploadError {
    fn into_response(self) -> Response {
        use UploadError::*;
        let default_err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
        let default_response =
            (StatusCode::INTERNAL_SERVER_ERROR, Json(default_err_msg)).into_response();
        match self {
            Database(_) | FailedToEnqueueTask(_) | NotSupported => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            DataFieldUnavailable(_)
            | IdMismatch
            | InvalidCid
            | DataFieldMissing
            | InvalidRequestData(_)
            | RequestFieldUnavailable(_)
            | MismatchedCid(_)
            | RequestFieldMissing => {
                let err_msg = serde_json::json!({ "msg": format!("{self}") });
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            InsufficientAuthorizedStorage(requested_bytes, remaining_bytes) => {
                tracing::warn!(upload_size = ?requested_bytes, remaining_storage = ?remaining_bytes, "user doesn't have sufficient storage capacity remaining");
                let err_msg = serde_json::json!({ "msg": format!("{self}") });
                (StatusCode::PAYLOAD_TOO_LARGE, Json(err_msg)).into_response()
            }
            ReadFailed(_) => {
                let err_msg = serde_json::json!({
                    "msg": format!("client stream went away before file upload was complete")
                });
                (StatusCode::UNPROCESSABLE_ENTITY, Json(err_msg)).into_response()
            }
            ObjectStore(err) => {
                tracing::error!("writing car file failed: {err}");
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            CarFile => {
                tracing::error!("client asked to write block to car");
                default_response
            }
            UploadLookupFailure => {
                let err_msg = serde_json::json!({ "msg": "upload not found" });
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            }
            UploadIsComplete => {
                tracing::warn!("client is trying to write more data to a completed upload");
                default_response
            }
            ParseError(err) => err.into_response(),
        }
    }
}

impl From<sqlx::Error> for UploadError {
    fn from(value: sqlx::Error) -> Self {
        UploadError::Database(map_sqlx_error(value))
    }
}
