use std::collections::BTreeSet;
use std::str::FromStr;

use axum::extract::{BodyStream, Path, State};
use axum::headers::ContentType;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Json, TypedHeader};
use futures::{TryStream, TryStreamExt};
use jwt_simple::prelude::*;
use mime::Mime;
use object_store::ObjectStore;
use serde::Deserialize;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use uuid::Uuid;

use crate::app::AppState;
use crate::auth::storage_ticket::StorageTicketBuilder;
use crate::database::models::{
    Bucket, Metadata, MetadataState, NewMetadata, NewStorageGrant, PendingExpiration,
    SelectedStorageHost, StorageHost, Subscription, User, UserStorageReport,
};
use crate::extractors::{ApiIdentity, DataStore};
use crate::utils::car_buffer::CarBuffer;
use crate::{utils, GIBIBYTE};

/// Size limit of the pure metadata CAR file that is being uploaded (128MiB)
const CAR_DATA_SIZE_LIMIT: u64 = 128 * 1_024 * 1_024;

/// The "official" mime type registered for CAR files, we specifically only accept version 2
const CAR_MIME_TYPE: &str = "application/vnd.ipld.car; version=2";

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
    let span = tracing::info_span!(
        "push_metadata_handler",
        bucket.id = %bucket_id,
        user.id = %api_id.user_id(),
    );
    let _guard = span.enter();

    let bucket_id = bucket_id.to_string();
    let user_id = api_id.user_id().to_string();

    let database = state.database();
    let mut conn = database.begin().await?;

    if !Bucket::is_owned_by_user_id(&mut conn, &bucket_id, &user_id).await? {
        let err_msg = serde_json::json!({"msg": "not found"});
        return Ok((StatusCode::NOT_FOUND, Json(err_msg)).into_response());
    }

    if Bucket::is_change_in_progress(&mut conn, &bucket_id).await? {
        tracing::warn!("attempted upload to bucket while other write was in progress");
        let err_msg = serde_json::json!({"msg": "waiting for other upload to complete"});
        return Ok((StatusCode::CONFLICT, Json(err_msg)).into_response());
    }

    // Request is authorized, and we're ready to receive it. Start processing the multipart
    // segments of the request.

    let mime_ct = mime::Mime::from(content_type);
    let boundary = multer::parse_boundary(mime_ct)?;

    let constraints = multer::Constraints::new()
        .allowed_fields(vec!["request-data", "car-upload"])
        .size_limit(
            multer::SizeLimit::new()
                .for_field("request-data", REQUEST_DATA_SIZE_LIMIT)
                .for_field("car-upload", CAR_DATA_SIZE_LIMIT),
        );
    let mut multipart = multer::Multipart::with_constraints(body, boundary, constraints);

    // First multipart is the uploaded metadata request details

    let request_field = match multipart.next_field().await? {
        Some(f) => f,
        None => {
            tracing::warn!("upload contained no request details");
            let err_msg = serde_json::json!({"msg": "missing request details"});
            return Ok((StatusCode::BAD_REQUEST, Json(err_msg)).into_response());
        }
    };

    if !validate_field(&request_field, "request-data", &mime::APPLICATION_JSON) {
        let err_msg = serde_json::json!({"msg": "request field is invalid"});
        return Ok((StatusCode::BAD_REQUEST, Json(err_msg)).into_response());
    }

    let request_data_bytes = request_field.bytes().await?;
    let request_data: PushMetadataRequest = match serde_json::from_slice(&request_data_bytes) {
        Ok(d) => d,
        Err(_) => {
            let err_msg = serde_json::json!({"msg": "request data was not a valid JSON object"});
            return Ok((StatusCode::BAD_REQUEST, Json(err_msg)).into_response());
        }
    };

    // If the client specifies a previous_cid, attempt to determine if its reachable
    // within the bucket's history following the most current version
    if let Some(previous_metadata_cid) = request_data.previous_cid {
        // If the update is not valid within the bucket's history, reject the request
        if !Bucket::update_is_valid(&mut conn, &bucket_id, &previous_metadata_cid).await? {
            tracing::warn!("pushed metadata specified conflicting previous metadata cid");
            let err_msg = serde_json::json!({"msg": "request specifies a previous_cid in conflict with the current history"});
            return Ok((StatusCode::CONFLICT, Json(err_msg)).into_response());
        }
    } else {
        tracing::warn!("pushed metadata specified no previous cid");
    };

    Bucket::approve_keys_by_fingerprint(
        &mut conn,
        &bucket_id,
        &request_data.included_key_fingerprints,
    )
    .await?;

    let metadata_id = NewMetadata {
        bucket_id: &bucket_id,

        metadata_cid: &request_data.metadata_cid,

        root_cid: &request_data.root_cid,
        expected_data_size: request_data.expected_data_size,
    }
    .save(&mut conn)
    .await?;

    let normalized_cids: Vec<_> = match request_data
        .deleted_block_cids
        .iter()
        .map(String::as_str)
        .map(utils::normalize_cid)
        .collect()
    {
        Ok(nc) => nc,
        Err(_) => {
            let err_msg = serde_json::json!({"msg": "request data included invalid CID in deleted block list"});
            return Ok((StatusCode::BAD_REQUEST, Json(err_msg)).into_response());
        }
    };

    PendingExpiration::record_pending_block_expirations(
        &mut conn,
        &bucket_id,
        &metadata_id,
        &normalized_cids,
    )
    .await?;

    // Checkpoint the upload to the database so we can track failures, and perform any necessary
    // clean up behind the scenes. The upload itself will also dwarf the rest of the time of this
    // request, limiting the time in those transactions is a good idea.
    conn.commit().await?;

    // Begin work on the second portion of the multipart request which is the raw data payload for
    // the encrypted metadata from the client. This is effectively our new FS version, but some
    // additional book keeping and data transfer needs to occur before we can mark this as active.

    let data_field = match multipart.next_field().await? {
        Some(d) => d,
        None => {
            tracing::warn!("upload contained no data");
            let err_msg = serde_json::json!({"msg": "missing request data"});
            return Ok((StatusCode::BAD_REQUEST, Json(err_msg)).into_response());
        }
    };

    let car_mime = Mime::from_str(CAR_MIME_TYPE).expect("static mime validated");
    if !validate_field(&data_field, "car-upload", &car_mime) {
        let err_msg = serde_json::json!({"msg": "upload data is unexpected type"});
        return Ok((StatusCode::BAD_REQUEST, Json(err_msg)).into_response());
    }

    let file_name = format!("{bucket_id}/{metadata_id}.car");
    let (hash, size) = persist_upload(&store, &file_name, data_field).await?;

    // We don't need to be in a tranaction yet, a regular acquire is fine here
    let mut conn = database.acquire().await?;
    Metadata::upload_complete(&mut conn, &metadata_id, &hash, size as i64).await?;

    let needed_capacity = request_data.expected_data_size;
    let user = User::by_id(&mut conn, &user_id).await?;
    let subscription = Subscription::by_id(&mut conn, &user.subscription_id).await?;

    if let Some(hard_limit) = subscription.hot_storage_hard_limit {
        let hard_limit_bytes = hard_limit * GIBIBYTE;

        let consumed_storage = user.consumed_storage(&mut conn).await?;

        if (consumed_storage + needed_capacity) > hard_limit_bytes {
            tracing::warn!(consumed_storage, "account reached storage limit");
            let err_msg = serde_json::json!({"msg": "account reached available storage threshold"});
            return Ok((StatusCode::PAYLOAD_TOO_LARGE, Json(err_msg)).into_response());
        }
    }

    if request_data.expected_data_size == 0 {
        Metadata::mark_current(&mut conn, &bucket_id, &metadata_id, None).await?;
        let resp_msg = serde_json::json!({"id": metadata_id, "state": "current"});
        return Ok((StatusCode::OK, Json(resp_msg)).into_response());
    }

    // We need to be consistent again, close and switch back to transaction land
    conn.close().await?;
    let mut conn = database.begin().await?;

    let storage_host =
        match SelectedStorageHost::select_for_capacity(&mut conn, needed_capacity, None).await? {
            Some(sh) => sh,
            None => {
                tracing::warn!(
                    needed_capacity,
                    "unable to locate host with sufficient capacity"
                );
                let err_msg = serde_json::json!({"msg": ""});
                return Ok((StatusCode::INSUFFICIENT_STORAGE, Json(err_msg)).into_response());
            }
        };
    let user_report = StorageHost::user_report(&mut conn, &storage_host.id, &user_id).await?;

    let mut storage_authorization: Option<String> = None;
    if user_report.authorization_available() < needed_capacity {
        let new_authorized_capacity = rounded_storage_authorization(&user_report, needed_capacity);

        let storage_grant_id = NewStorageGrant {
            storage_host_id: &storage_host.id,
            user_id: &user_id,
            authorized_amount: new_authorized_capacity,
        }
        .save(&mut conn)
        .await?;

        let mut ticket_builder = StorageTicketBuilder::new(api_id.ticket_subject());
        ticket_builder.add_audience(storage_host.name);
        ticket_builder.add_authorization(
            storage_grant_id,
            storage_host.url.clone(),
            new_authorized_capacity,
        );

        let claim = ticket_builder.build();

        let service_key = state.secrets().service_key();
        let ticket = match service_key.sign(claim) {
            Ok(t) => t,
            Err(err) => {
                tracing::error!("failed to sign storage authorization: {err}");
                let err_msg = serde_json::json!({"msg": "authorization delegation unavailable"});
                return Ok((StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response());
            }
        };

        storage_authorization = Some(ticket);
    }

    conn.commit().await?;

    let response = serde_json::json!({
        "id": metadata_id,
        "state": MetadataState::Pending,
        "storage_host": storage_host.url,
        "storage_authorization": storage_authorization,
    });

    Ok((StatusCode::OK, Json(response)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum PushMetadataError {
    #[error("failed to run query: {0}")]
    QueryFailure(#[from] sqlx::Error),

    #[error("the request was badly formatted: {0}")]
    InvalidMultipart(#[from] multer::Error),

    #[error("failed to persist upload: {0}")]
    PersistanceFailure(#[from] PersistanceError),
}

impl IntoResponse for PushMetadataError {
    fn into_response(self) -> Response {
        tracing::error!("internal error handling metadata upload: {self}");
        let err_msg = serde_json::json!({"msg": "a backend service issue encountered an error"});
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
    }
}

async fn persist_upload<'a>(
    store: &DataStore,
    path: &str,
    body: multer::Field<'a>,
) -> Result<(String, usize), PersistanceError> {
    let file_path = object_store::path::Path::from(path);
    let (upload_id, mut writer) = store.put_multipart(&file_path).await?;

    let (hash, size) = match stream_upload_to_storage(body, &mut writer).await {
        Ok(out) => out,
        Err(err) => {
            // This abort handles clean-up of stored files, if it fails it can be cleaned up in the
            // background elsewhere. Ensure we return the error that is most relevant to the
            // user/process.
            let _ = store.abort_multipart(&file_path, &upload_id).await;
            return Err(err);
        }
    };

    writer.shutdown().await?;

    Ok((hash, size))
}

pub fn rounded_storage_authorization(report: &UserStorageReport, additional_capacity: i64) -> i64 {
    let new_required_amount = report.current_consumption() + additional_capacity;
    // Integer division always rounds down, we want to round up to the nearest 100MiB
    ((new_required_amount / ONE_HUNDRED_MIB) + 1) * ONE_HUNDRED_MIB
}

async fn stream_upload_to_storage<S>(
    mut stream: S,
    writer: &mut Box<dyn AsyncWrite + Unpin + Send>,
) -> Result<(String, usize), PersistanceError>
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
        .map_err(|err| PersistanceError::StreamFailure(err.to_string()))?
    {
        hasher.update(&chunk);
        car_buffer.add_chunk(&chunk);
        bytes_written += chunk.len();

        writer.write_all(&chunk).await?;
    }

    let hash = hasher.finalize();

    Ok((hash.to_string(), bytes_written))
}

/// Validates that a particular multipart field has the expected name and content type. Currently
/// this only warns when they are mismatched as some of our clients aren't as well behaved as
/// others. Once our official clients no longer generate warnings this should start rejecting
/// invalid requests.
#[tracing::instrument(skip(field))]
fn validate_field(
    field: &multer::Field,
    expected_name: &str,
    expected_content_type: &mime::Mime,
) -> bool {
    let field_name = field.name();
    if field_name != Some(expected_name) {
        tracing::warn!(field_name, "field name didn't match expected");
    }

    let field_content_type = field.content_type();
    if field_content_type != Some(expected_content_type) {
        tracing::warn!(
            ?field_content_type,
            "field content type didn't match expected"
        );
    }

    true
}

#[derive(Debug, thiserror::Error)]
pub enum PersistanceError {
    #[error("an I/O error occurred while writing metadata: {0}")]
    Io(#[from] std::io::Error),

    #[error("upload library encountered setup error: {0}")]
    StoreError(#[from] object_store::Error),

    #[error("failure in client stream: {0}")]
    StreamFailure(String),
}

#[derive(Deserialize)]
pub struct PushMetadataRequest {
    // TODO: either let's remove the distinction between root and metadata cids
    // or rename this to previous_metadata_cid. For now the client as implemented
    // within the cli / wasm is expecting this name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_cid: Option<String>,

    pub root_cid: String,
    pub metadata_cid: String,

    pub expected_data_size: i64,

    /// Fingerprints of the public portion of the bucket keys that are valid for this metadata
    /// upload
    #[serde(rename = "valid_keys")]
    pub included_key_fingerprints: Vec<String>,

    pub deleted_block_cids: BTreeSet<String>,
}
