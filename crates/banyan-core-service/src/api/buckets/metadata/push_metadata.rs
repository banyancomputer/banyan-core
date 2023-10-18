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
const ACCOUNT_STORAGE_QUOTA: i64 = 5 * 1_024 * 1_024 * 1_024 * 1_024;

/// Size limit of the pure metadata CAR file that is being uploaded (128MiB)
const CAR_DATA_SIZE_LIMIT: u64 = 128 * 1_024 * 1_024;

const ONE_HUNDRED_MIB: i64 = 100 * 1024 * 1024;

/// Upper size limit on the JSON payload that precedes a metadata CAR file upload (128KiB)
const REQUEST_DATA_SIZE_LIMIT: u64 = 128 * 1_024;

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
    let authorized_bucket_id =
        authorized_bucket_id(&database, &api_id.account_id, &unvalidated_bid).await?;

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
        &request_data.included_key_fingerprints,
    )
    .await?;

    let new_metadata_id =
        record_upload_start(&database, &authorized_bucket_id, &request_data).await?;

    let data_body = multipart
        .next_field()
        .await
        .map_err(PushMetadataError::BrokenMultipartField)?
        .ok_or(PushMetadataError::MissingMetadata)?;

    // TODO: validate name is car-upload (request_data_field.name())
    // TODO: validate type is "application/vnd.ipld.car; version=2" (request_data_field.content_type())
    let file_name = format!("{authorized_bucket_id}/{new_metadata_id}.car");
    let (hash, uploaded_size) =
        match store_metadata_stream(&store, file_name.as_str(), data_body).await {
            Ok(mhs) => mhs,
            Err(store_err) => {
                let fail_update_res = fail_upload(&database, &new_metadata_id).await.err();
                return Err(PushMetadataError::UploadStoreFailed(
                    store_err,
                    fail_update_res,
                ));
            }
        };

    record_data_stored(&database, &new_metadata_id, uploaded_size, &hash)
        .await
        .map_err(PushMetadataError::DataMetaStoreFailed)?;

    let currently_consumed_storage = currently_consumed_storage(&database, &api_id.account_id)
        .await
        .map_err(PushMetadataError::UnableToCheckAccounting)?;
    let expected_total_storage = currently_consumed_storage as i64 + request_data.expected_data_size;

    if expected_total_storage > ACCOUNT_STORAGE_QUOTA {
        tracing::warn!(account_id = ?api_id.account_id, ?expected_total_storage, "account reached storage limit");
        let fail_res = fail_upload(&database, &new_metadata_id).await;
        return Err(PushMetadataError::LimitReached(fail_res.err()));
    }

    if request_data.expected_data_size == 0 {
        mark_current(&database, &new_metadata_id)
            .await
            .map_err(PushMetadataError::ActivationFailed)?;

        let resp_msg = serde_json::json!({"id": new_metadata_id, "state": "current"});
        return Ok((StatusCode::OK, Json(resp_msg)).into_response());
    }

    let expected_data_size = ((request_data.expected_data_size / ONE_HUNDRED_MIB) + 1) * ONE_HUNDRED_MIB;
    let storage_host = select_storage_host(&database, expected_data_size).await?;

    todo!()
}

async fn select_storage_host(database: &Database, required_space: i64) -> Result<SelectedStorageHost, PushMetadataError> {
    let maybe_storage_host = sqlx::query_as!(
        SelectedStorageHost,
        r#"SELECT id, name, url FROM storage_hosts
               WHERE (available_storage - used_storage) > $1
               ORDER BY RANDOM()
               LIMIT 1;"#,
        required_space,
    )
    .fetch_optional(database)
    .await
    .map_err(PushMetadataError::StorageHostLookupFailed)?;

    match maybe_storage_host {
        Some(shi) => Ok(shi),
        None => {
            tracing::error!(?required_space, "failed to locate storage host with sufficient storage");
            Err(PushMetadataError::NoAvailableStorage)
        }
    }
}

async fn approve_key_fingerprints(
    database: &Database,
    bucket_id: &str,
    keys: &Vec<String>,
) -> Result<(), PushMetadataError> {
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

async fn authorized_bucket_id(
    database: &Database,
    account_id: &str,
    bucket_id: &str,
) -> Result<String, PushMetadataError> {
    sqlx::query_scalar!(
        r#"SELECT id FROM buckets WHERE account_id = $1 AND id = $2;"#,
        account_id,
        bucket_id,
    )
    .fetch_optional(database)
    .await
    .map_err(PushMetadataError::BucketAuthorizationFailed)?
    .ok_or(PushMetadataError::NoAuthorizedBucket)
}

async fn currently_consumed_storage(database: &Database, account_id: &str) -> Result<i32, sqlx::Error> {
    let maybe_stored = sqlx::query_scalar!(
        r#"SELECT
            COALESCE(SUM(m.metadata_size), 0) +
            COALESCE(SUM(COALESCE(m.data_size, m.expected_data_size)), 0)
        FROM
            metadata m
        INNER JOIN
            buckets b ON b.id = m.bucket_id
        WHERE
            b.account_id = $1 AND m.state IN ('current', 'outdated', 'pending');"#,
        account_id,
    )
    .fetch_optional(database)
    .await?;

    Ok(maybe_stored.unwrap_or(0))
}

async fn mark_current(database: &Database, metadata_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE metadata SET state = 'current' WHERE id = $1",
        metadata_id,
    )
    .execute(database)
    .await?;

    sqlx::query!(
        "UPDATE metadata SET state = 'outdated' WHERE id != $1 AND state = 'current';",
        metadata_id,
    )
    .execute(database)
    .await?;

    Ok(())
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

async fn record_data_stored(
    database: &Database,
    metadata_id: &str,
    uploaded_size: usize,
    data_hash: &str,
) -> Result<(), sqlx::Error> {
    let db_size = uploaded_size as i32;

    sqlx::query!(
        "UPDATE metadata SET metadata_size = $2, metadata_hash = $3 WHERE id = $1;",
        metadata_id,
        db_size,
        data_hash,
    )
    .execute(database)
    .await?;

    // todo: if this fails really need to clean up the files and everything... good task for a
    // background job...

    Ok(())
}

async fn record_upload_start(
    database: &Database,
    bucket_id: &str,
    request: &PushMetadataRequest,
) -> Result<String, PushMetadataError> {
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
pub enum PushMetadataError {
    #[error("failed updating zero data metadata to current")]
    ActivationFailed(sqlx::Error),

    #[error("failed to validate bucket was authorized by user: {0}")]
    BucketAuthorizationFailed(sqlx::Error),

    #[error("unable to pull next multipart field: {0}")]
    BrokenMultipartField(multer::Error),

    #[error("failed to record metadata size and hash: {0}")]
    DataMetaStoreFailed(sqlx::Error),

    #[error("failed to mark bucket key as approved: {0}")]
    KeyApprovalFailed(sqlx::Error),

    #[error("failed to create entry for metadata in the database: {0}")]
    MetadataRegistrationFailed(sqlx::Error),

    #[error("unable to parse valid boundary: {0}")]
    InvalidBoundary(multer::Error),

    #[error("provided request data couldn't be decoded: {0}")]
    InvalidRequestData(serde_json::Error),

    #[error("account reached upload quota and recording the failure may have failed: {0:?}")]
    LimitReached(Option<sqlx::Error>),

    #[error("request did not contain required data segment")]
    MissingRequestData,

    #[error("request did not contain required metadata segment")]
    MissingMetadata,

    #[error("unable to locate a bucket for the current authorized user")]
    NoAuthorizedBucket,

    #[error("no storage host is available with sufficient storage")]
    NoAvailableStorage,

    #[error("unable to retrieve request data: {0}")]
    RequestDataUnavailable(multer::Error),

    #[error("failed to query for available storage host: {0}")]
    StorageHostLookupFailed(sqlx::Error),

    #[error("unable to determine if user is within their quota: {0}")]
    UnableToCheckAccounting(sqlx::Error),

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
            PushMetadataError::LimitReached(_) => {
                let err_msg = serde_json::json!({"msg": "you have hit your account storage limit"});
                (StatusCode::PAYLOAD_TOO_LARGE, Json(err_msg)).into_response()
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

#[derive(sqlx::FromRow)]
struct SelectedStorageHost {
    pub id: String,
    pub name: String,
    pub url: String,
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
pub enum StreamStoreError {
    #[error("failed to retrieve next expected chunk: {0}")]
    NeedChunk(String),

    #[error("failed to write out chunk: {0}")]
    WriteFailed(std::io::Error),
}
