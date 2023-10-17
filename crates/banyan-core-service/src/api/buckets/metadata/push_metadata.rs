use axum::body::StreamBody;
use axum::extract::{BodyStream, Path, State};
use axum::headers::ContentType;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use axum::TypedHeader;
use http::{HeaderMap, HeaderValue};
use object_store::ObjectStore;
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

    //let request_data: requests::PushMetadataRequest = match serde_json::from_slice(&request_data_bytes) {
    //    Ok(rdb) => rdb,
    //    Err(err) => {
    //        return (
    //            StatusCode::BAD_REQUEST,
    //            Json(serde_json::json!({"msg": format!("{err}")})),
    //            )
    //            .into_response();
    //    }
    //};

    todo!()
}

#[derive(Debug, thiserror::Error)]
pub enum PushMetadataError {
    #[error("failed to validate bucket was authorized by user: {0}")]
    BucketAuthorizationFailed(sqlx::Error),

    #[error("unable to pull next multipart field: {0}")]
    BrokenMultipartField(multer::Error),

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
