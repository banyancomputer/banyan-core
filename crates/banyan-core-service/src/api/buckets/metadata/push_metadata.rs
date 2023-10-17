use axum::body::StreamBody;
use axum::extract::{BodyStream, Path, State};
use axum::headers::ContentType;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use axum::TypedHeader;
use futures::{TryStream, TryStreamExt};
use http::{HeaderMap, HeaderValue};
use object_store::ObjectStore;
use serde::Deserialize;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use uuid::Uuid;

use crate::app::AppState;
use crate::database::Database;
use crate::extractors::{ApiIdentity, DataStore, SigningKey};
use crate::utils::car_buffer::CarBuffer;
//use crate::utils::metadata_upload::{round_to_nearest_100_mib};
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

    approve_key_fingerprints(
        &database,
        &authorized_bucket_id,
        &request_data.included_key_fingerprints
    ).await?;

    let new_metadata_id = record_upload_start(&database, &authorized_bucket_id, &request_data).await?;

    let car_stream = multipart
        .next_field()
        .await
        .map_err(PushMetadataError::BrokenMultipartField)?
        .ok_or(PushMetadataError::MissingMetadata)?;

    // TODO: validate name is car-upload (request_data_field.name())
    // TODO: validate type is "application/vnd.ipld.car; version=2" (request_data_field.content_type())
    let file_name = format!("{authorized_bucket_id}/{new_metadata_id}.car");
    let (metadata_hash, metadata_size) =
        match store_metadata_stream(&store, file_name.as_str(), car_stream).await {
            Ok(mhs) => mhs,
            Err(store_err) => {
                return Err(PushMetadataError::UploadStoreFailed(
                    store_err,
                    fail_upload(&database, &new_metadata_id).await.err(),
                ));
            }
        };

    todo!()
}

async fn approve_key_fingerprints(database: &Database, bucket_id: &str, keys: &Vec<String>) -> Result<(), PushMetadataError> {
    for device_key_fingerprint in keys.iter() {
        sqlx::query!(
            "UPDATE bucket_keys SET approved = 'true' WHERE bucket_id = $1 AND fingerprint = $2;",
            bucket_id,
            device_key_fingerprint,
        )
        .execute(database)
        .await
        .map_err(PushMetadataError::KeyApprovalFailed)?;
    }

    Ok(())
}

async fn record_upload_start(database: &Database, bucket_id: &str, request: &PushMetadataRequest) -> Result<String, PushMetadataError> {
    sqlx::query_scalar!(
        r#"INSERT INTO metadata (bucket_id, root_cid, metadata_cid, expected_data_size, state)
               VALUES ($1, $2, $3, $4, 'uploading')
               RETURNING id;"#,
        bucket_id,
        request.root_cid,
        request.metadata_cid,
        request.expected_data_size,
    )
    .fetch_one(database)
    .await
    .map_err(PushMetadataError::MetadataRegistrationFailed)
}

async fn fail_upload(database: &Database, metadata_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE metadata SET state = 'upload_failed' WHERE id = $1",
        metadata_id,
    )
    .execute(database)
    .await?;

    Ok(())
}

async fn store_metadata_stream<'a>(
    store: &DataStore,
    path: &str,
    mut body: multer::Field<'a>,
) -> Result<(String, usize), StoreMetadataError> {
    let file_path = object_store::path::Path::from(path);

    let (upload_id, mut writer) = store
        .put_multipart(&file_path)
        .await
        .map_err(StoreMetadataError::PutFailed)?;

    match stream_upload_to_storage(body, &mut writer).await {
        Ok(store_output) => {
            writer
                .shutdown()
                .await
                .map_err(StoreMetadataError::FinalizationFailed)?;
            Ok(store_output)
        }
        Err(err) => {
            let abort_res = store.abort_multipart(&file_path, &upload_id).await;

            Err(StoreMetadataError::StreamingFailed(err, abort_res.err()))
        }
    }
}

async fn stream_upload_to_storage<S>(
    mut stream: S,
    writer: &mut Box<dyn AsyncWrite + Unpin + Send>,
) -> Result<(String, usize), StreamStoreError>
where
    S: TryStream<Ok = bytes::Bytes> + Unpin,
    S::Error: std::error::Error,
{
    let mut car_buffer = CarBuffer::new();
    let mut hasher = blake3::Hasher::new();
    let mut bytes_written = 0;

    while let Some(chunk) = stream
        .try_next()
        .await
        .map_err(|err| StreamStoreError::NeedChunk(err.to_string()))?
    {
        hasher.update(&chunk);
        car_buffer.add_chunk(&chunk);
        bytes_written += chunk.len();

        writer
            .write_all(&chunk)
            .await
            .map_err(StreamStoreError::WriteFailed)?;
    }

    let hash = hasher.finalize();

    Ok((hash.to_string(), bytes_written))
}

#[derive(Debug, thiserror::Error)]
pub enum StreamStoreError {
    #[error("failed to retrieve next expected chunk: {0}")]
    NeedChunk(String),

    #[error("failed to write out chunk: {0}")]
    WriteFailed(std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum StoreMetadataError {
    #[error("failed to finalize storage to disk: {0}")]
    FinalizationFailed(std::io::Error),

    #[error("failed to begin file write transaction: {0}")]
    PutFailed(object_store::Error),

    #[error("failed to stream upload to storage: {0}, aborting might have also failed: {1:?}")]
    StreamingFailed(StreamStoreError, Option<object_store::Error>),
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

    #[error("request did not contain required metadata segment")]
    MissingMetadata,

    #[error("unable to locate a bucket for the current authorized user")]
    NoAuthorizedBucket,

    #[error("unable to retrieve request data: {0}")]
    RequestDataUnavailable(multer::Error),

    #[error("failed to store metadata on disk: {0}, marking as failed might have had an error as well: {1:?}")]
    UploadStoreFailed(StoreMetadataError, Option<sqlx::Error>),
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
