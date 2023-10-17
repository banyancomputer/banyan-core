use axum::body::StreamBody;
use axum::extract::{BodyStream, Path, State};
use axum::headers::ContentType;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use axum::TypedHeader;
use http::{HeaderMap, HeaderValue};
use object_store::ObjectStore;
use serde::Deserialize;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::app::AppState;
use crate::database::Database;
use crate::extractors::{ApiIdentity, DataStore, SigningKey};
//use crate::utils::metadata_upload::{handle_metadata_upload, round_to_nearest_100_mib};
//use crate::utils::storage_ticket::generate_storage_ticket;

/// The default quota we assume each storage host / staging service to provide
const ACCOUNT_STORAGE_QUOTA: u64 = 5 * 1_024 * 1_024 * 1_024 * 1_024;

/// Upper size limit on the JSON payload that precedes a metadata CAR file upload (128KiB)
const REQUEST_DATA_SIZE_LIMIT: u64 = 128 * 1_024;

/// Size limit of the pure metadata CAR file that is being uploaded (128MiB)
const CAR_DATA_SIZE_LIMIT: u64 = 128 * 1_024 * 1_024;

pub async fn handler(
    api_id: ApiIdentity,
    State(state): State<AppState>,
    store: DataStore,
    Path(bucket_id): Path<Uuid>,
    TypedHeader(content_type): TypedHeader<ContentType>,
    body: BodyStream,
) -> Result<Response, PushMetadataError> {
    let database = state.database();
    let service_signing_key = state.secrets().service_signing_key();

    let unvalidated_bid = bucket_id.to_string();

    let maybe_bucket_id = sqlx::query_scalar!(
        r#"SELECT id FROM buckets WHERE account_id = $1 AND id = $2;"#,
        api_id.account_id,
        unvalidated_bid,
    )
    .fetch_optional(&database)
    .await
    .map_err(PushMetadataError::BucketAuthorizationFailed)?;

    let authorized_bucket_id = match maybe_bucket_id {
        Some(abi) => abi,
        None => return Err(PushMetadataError::NoAuthorizedBucket),
    };

    // Read the body from the request, checking for size limits
    let mime_ct = mime::Mime::from(content_type);
    let boundary = multer::parse_boundary(mime_ct).map_err(PushMetadataError::InvalidBoundary)?;

    let constraints = multer::Constraints::new()
        .allowed_fields(vec!["request-data", "car-upload"])
        .size_limit(
            multer::SizeLimit::new()
                .for_field("request-data", REQUEST_DATA_SIZE_LIMIT)
                .for_field("car-upload", CAR_DATA_SIZE_LIMIT),
        );
    let mut multipart = multer::Multipart::with_constraints(body, boundary, constraints);

    // Process the request data
    let request_data_field = multipart
        .next_field()
        .await
        .map_err(PushMetadataError::BrokenMultipartField)?
        .ok_or(PushMetadataError::MissingRequestData)?;

    // todo: validate name is request-data (request_data_field.name())
    // todo: validate type is application/json (request_data_field.content_type())
    let request_data_bytes = request_data_field
        .bytes()
        .await
        .map_err(PushMetadataError::RequestDataUnavailable)?;

    let request_data: PushMetadataRequest = serde_json::from_slice(&request_data_bytes)
        .map_err(PushMetadataError::InvalidRequestData)?;

    for device_key_fingerprint in request_data.included_key_fingerprints.iter() {
        let query_res = sqlx::query!(
            "UPDATE bucket_keys SET approved = 'true' WHERE bucket_id = $1 AND fingerprint = $2;",
            authorized_bucket_id,
            device_key_fingerprint,
        )
        .execute(&database)
        .await
        .map_err(PushMetadataError::KeyApprovalFailed)?;
    }

    let new_metadata_id = sqlx::query_scalar!(
        r#"INSERT INTO metadata (bucket_id, root_cid, metadata_cid, expected_data_size, state)
               VALUES ($1, $2, $3, $4, 'uploading')
               RETURNING id;"#,
        authorized_bucket_id,
        request_data.root_cid,
        request_data.metadata_cid,
        request_data.expected_data_size,
    )
    .fetch_one(&database)
    .await
    .map_err(PushMetadataError::MetadataRegistrationFailed)?;

    todo!()
}

#[derive(Debug, thiserror::Error)]
pub enum PushMetadataError {
    #[error("failed to validate bucket was authorized by user: {0}")]
    BucketAuthorizationFailed(sqlx::Error),

    #[error("unable to pull next multipart field: {0}")]
    BrokenMultipartField(multer::Error),

    #[error("failed to mark bucket key as approved: {0}")]
    KeyApprovalFailed(sqlx::Error),

    #[error("failed to create entry for metadata in the database: {0}")]
    MetadataRegistrationFailed(sqlx::Error),

    #[error("unable to parse valid boundary: {0}")]
    InvalidBoundary(multer::Error),

    #[error("provided request data couldn't be decoded: {0}")]
    InvalidRequestData(serde_json::Error),

    #[error("request did not contain required data segment")]
    MissingRequestData,

    #[error("unable to locate a bucket for the current authorized user")]
    NoAuthorizedBucket,

    #[error("unable to retrieve request data: {0}")]
    RequestDataUnavailable(multer::Error),
}

impl IntoResponse for PushMetadataError {
    fn into_response(self) -> Response {
        match &self {
            PushMetadataError::BrokenMultipartField(_)
            | PushMetadataError::InvalidBoundary(_)
            | PushMetadataError::InvalidRequestData(_)
            | PushMetadataError::MissingRequestData => {
                let err_msg = serde_json::json!({"msg": "invalid request"});
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            PushMetadataError::NoAuthorizedBucket => {
                let err_msg = serde_json::json!({"msg": "not found"});
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            }
            _ => {
                tracing::error!("failed to push metadata: {self}");
                let err_msg = serde_json::json!({"msg": "an internal server error occurred"});
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            }
        }
    }
}

#[derive(Deserialize)]
pub struct PushMetadataRequest {
    pub root_cid: String,
    pub metadata_cid: String,

    pub expected_data_size: i64,

    #[serde(rename = "valid_keys")]
    pub included_key_fingerprints: Vec<String>,
}
