use std::collections::{BTreeSet, HashMap, HashSet};
use std::str::FromStr;

use axum::extract::{BodyStream, Path, State};
use axum::headers::ContentType;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Json, TypedHeader};
use banyan_task::TaskLikeExt;
use cid::{multibase::Base, Cid};
use futures::{TryStream, TryStreamExt};
use jwt_simple::prelude::*;
use object_store::ObjectStore;
use serde::Deserialize;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use uuid::Uuid;

use crate::app::{AppState, ServiceSigningKey};
use crate::database::models::MetadataState;
use crate::database::Database;
use crate::extractors::{ApiIdentity, DataStore};
use crate::tasks::{PruneBlock, PruneBlocksTask};
use crate::utils::car_buffer::CarBuffer;

/// The default quota we assume each storage host / staging service to provide
const ACCOUNT_STORAGE_QUOTA: i64 = 5 * 1_024 * 1_024 * 1_024 * 1_024;

/// Size limit of the pure metadata CAR file that is being uploaded (128MiB)
const CAR_DATA_SIZE_LIMIT: u64 = 128 * 1_024 * 1_024;

const ONE_HUNDRED_MIB: i64 = 100 * 1024 * 1024;

/// Upper size limit on the JSON payload that precedes a metadata CAR file upload (128KiB)
const REQUEST_DATA_SIZE_LIMIT: u64 = 128 * 1_024;

pub const STORAGE_TICKET_DURATION: u64 = 15 * 60; // 15 minutes

pub async fn handler(
    api_id: ApiIdentity,
    State(state): State<AppState>,
    store: DataStore,
    Path(bucket_id): Path<Uuid>,
    TypedHeader(content_type): TypedHeader<ContentType>,
    body: BodyStream,
) -> Result<Response, PushMetadataError> {
    let mut database = state.database();
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

    // todo: validate name is car-upload (request_data_field.name())
    // todo: validate type is "application/vnd.ipld.car; version=2" (request_data_field.content_type())
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

    expire_deleted_blocks(&mut database, &api_id, &bucket_id, &request_data).await?;

    let expected_total_storage = currently_consumed_storage(&database, &api_id.account_id)
        .await
        .map_err(PushMetadataError::UnableToCheckAccounting)?
        as i64;

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

    let storage_host = select_storage_host(&database, request_data.expected_data_size).await?;

    let current_authorized_amount =
        existing_authorization(&database, &api_id.account_id, &storage_host.id)
            .await
            .map_err(PushMetadataError::UnableToRetrieveAuthorizations)?;
    let current_stored_amount =
        currently_stored_at_provider(&database, &api_id.account_id, &storage_host.id)
            .await
            .map_err(PushMetadataError::UnableToIdentifyStoredAmount)?;

    let mut storage_authorization: Option<String> = None;
    if (current_authorized_amount - current_stored_amount) < request_data.expected_data_size {
        let currently_required_amount = current_stored_amount + request_data.expected_data_size;
        let data_size_rounded =
            ((currently_required_amount / ONE_HUNDRED_MIB) + 1) * ONE_HUNDRED_MIB;

        let new_authorization = generate_new_storage_authorization(
            &database,
            &service_signing_key,
            &api_id,
            &storage_host,
            data_size_rounded,
        )
        .await
        .map_err(PushMetadataError::UnableToGenerateAuthorization)?;

        storage_authorization = Some(new_authorization);
    }

    let response = serde_json::json!({
        "id": new_metadata_id,
        "state": MetadataState::Pending,
        "storage_host": storage_host.url,
        "storage_authorization": storage_authorization,
    });

    Ok((StatusCode::OK, Json(response)).into_response())
}

async fn approve_key_fingerprints(
    database: &Database,
    bucket_id: &str,
    keys: &[String],
) -> Result<(), PushMetadataError> {
    for device_key_fingerprint in keys.iter() {
        sqlx::query!(
            "UPDATE bucket_keys SET approved = 1 WHERE bucket_id = $1 AND fingerprint = $2;",
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

async fn currently_consumed_storage(
    database: &Database,
    account_id: &str,
) -> Result<i32, sqlx::Error> {
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

async fn currently_stored_at_provider(
    database: &Database,
    account_id: &str,
    storage_host_id: &str,
) -> Result<i64, sqlx::Error> {
    let res: Result<Option<i64>, _> = sqlx::query_scalar!(
        r#"SELECT SUM(m.data_size) as total_data_size FROM metadata m
               JOIN storage_hosts_metadatas_storage_grants shmg ON shmg.metadata_id = m.id
               JOIN storage_grants sg ON shmg.storage_grant_id = sg.id
               WHERE sg.account_id = $1 AND shmg.storage_host_id = $2;"#,
        account_id,
        storage_host_id,
    )
    .fetch_one(database)
    .await;

    res.map(|amt| amt.unwrap_or(0))
}

async fn existing_authorization(
    database: &Database,
    account_id: &str,
    storage_host_id: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT authorized_amount FROM storage_grants
               WHERE account_id = $1
                   AND storage_host_id = $2
                   AND redeemed_at IS NOT NULL
               ORDER BY created_at DESC
               LIMIT 1;"#,
        account_id,
        storage_host_id,
    )
    .fetch_optional(database)
    .await
    .map(|amt| amt.unwrap_or(0))
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

#[derive(Debug, Serialize, Deserialize)]
struct Capabilities {
    #[serde(rename = "cap")]
    capabilities: serde_json::Map<String, serde_json::Value>,
}

async fn generate_new_storage_authorization(
    database: &Database,
    service_signing_key: &ServiceSigningKey,
    api_id: &ApiIdentity,
    storage_host: &SelectedStorageHost,
    authorized_amount: i64,
) -> Result<String, StorageAuthorizationError> {
    let storage_grant_id = sqlx::query_scalar!(
        r#"INSERT INTO storage_grants (storage_host_id, account_id, authorized_amount)
            VALUES ($1, $2, $3)
            RETURNING id;"#,
        storage_host.id,
        api_id.account_id,
        authorized_amount,
    )
    .fetch_one(database)
    .await
    .map_err(StorageAuthorizationError::GrantRecordingFailed)?;

    let mut storage_details = serde_json::Map::new();

    storage_details.insert("available_storage".to_string(), authorized_amount.into());
    storage_details.insert("grant_id".to_string(), storage_grant_id.into());

    let mut capabilities = serde_json::Map::new();
    capabilities.insert(storage_host.url.to_string(), storage_details.into());

    let mut claims = Claims::with_custom_claims(
        Capabilities { capabilities },
        Duration::from_secs(STORAGE_TICKET_DURATION),
    )
    .with_audiences(HashSet::from_strings(&[storage_host.name.as_str()]))
    .with_issuer("banyan-platform")
    .with_subject(format!(
        "{}@{}",
        api_id.user_id, api_id.device_api_key_fingerprint
    ))
    .invalid_before(Clock::now_since_epoch() - Duration::from_secs(30));

    claims.create_nonce();
    claims.issued_at = Some(Clock::now_since_epoch());

    let bearer_token = service_signing_key
        .sign(claims)
        .map_err(StorageAuthorizationError::SignatureFailed)?;

    Ok(bearer_token)
}

async fn mark_current(database: &Database, metadata_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE metadata SET state = 'current' WHERE id = $1",
        metadata_id,
    )
    .execute(database)
    .await?;

    sqlx::query!(
        "UPDATE metadata SET state = 'outdated' WHERE id <> $1 AND state = 'current';",
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
        "UPDATE metadata SET metadata_size = $2, metadata_hash = $3, state = 'pending' WHERE id = $1;",
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

async fn select_storage_host(
    database: &Database,
    required_space: i64,
) -> Result<SelectedStorageHost, PushMetadataError> {
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
            tracing::error!(
                ?required_space,
                "failed to locate storage host with sufficient storage"
            );
            Err(PushMetadataError::NoAvailableStorage)
        }
    }
}

async fn store_metadata_stream<'a>(
    store: &DataStore,
    path: &str,
    body: multer::Field<'a>,
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
                .map_err(StoreMetadataError::NotFinalized)?;
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

#[derive(sqlx::FromRow)]
struct UniqueBlockLocation {
    block_id: String,
    metadata_id: String,
    storage_host_id: String,
}

async fn expire_deleted_blocks(
    database: &mut Database,
    api_id: &ApiIdentity,
    bucket_id: &Uuid,
    request: &PushMetadataRequest,
) -> Result<(), PushMetadataError> {
    let account_id = api_id.account_id.clone();
    let bucket_id = bucket_id.to_string();
    let mut prune_blocks_tasks_map: HashMap<Uuid, Vec<PruneBlock>> = HashMap::new();
    let mut transaction = database
        .begin()
        .await
        .map_err(PushMetadataError::UnableToExpireBlocks)?;
    for original_cid in request.deleted_block_cids.clone() {
        let normalized_cid = Cid::from_str(&original_cid)
            .map_err(PushMetadataError::InvalidCid)?
            .to_string_of_base(Base::Base64Url)
            .map_err(PushMetadataError::InvalidCid)?;

        // Get all the unique blocks locations for this CID [(block_id, metadata_id, storage_host_id)]
        let unique_block_locations = sqlx::query_as!(
            UniqueBlockLocation,
            r#"SELECT blocks.id AS block_id, m.id AS metadata_id, sh.id AS storage_host_id
                FROM block_locations AS bl
                JOIN blocks ON blocks.id = bl.block_id
                JOIN storage_hosts AS sh ON sh.id = bl.storage_host_id
                JOIN metadata AS m ON m.id = bl.metadata_id
                JOIN buckets AS b ON b.id = m.bucket_id
                WHERE b.account_id = $1 AND b.id = $2 AND blocks.cid = $3;"#,
            account_id,
            bucket_id,
            normalized_cid,
        )
        .fetch_all(&mut *transaction)
        .await
        .map_err(PushMetadataError::UnableToExpireBlocks)?;

        if unique_block_locations.is_empty() {
            return Err(PushMetadataError::NoBlock(original_cid));
        }

        // Collect all the unique blocks into a single set
        let mut unique_blocks: BTreeSet<(String, String)> = BTreeSet::new();
        // Collect all the unique blocks into a map of prune blocks tasks for different storage hosts
        for unique_block_location in unique_block_locations {
            let unique_block = (
                unique_block_location.block_id.clone(),
                unique_block_location.metadata_id.clone(),
            );
            unique_blocks.insert(unique_block);
            let prune_block = PruneBlock {
                normalized_cid: normalized_cid.clone(),
                metadata_id: Uuid::parse_str(&unique_block_location.metadata_id)
                    .map_err(PushMetadataError::DatabaseUuidCorrupted)?,
            };
            let storage_host_id = Uuid::parse_str(&unique_block_location.storage_host_id)
                .map_err(PushMetadataError::DatabaseUuidCorrupted)?;
            prune_blocks_tasks_map
                .entry(storage_host_id)
                .or_default()
                .push(prune_block.clone());
        }

        // Iterate over the collected unique blocks and update their expired_at to now
        for unique_block in unique_blocks {
            // Update all the rows in place
            sqlx::query_scalar!(
                r#"UPDATE block_locations AS bl
                SET expired_at = CURRENT_TIMESTAMP
                WHERE bl.block_id = $1 AND bl.metadata_id = $2;"#,
                unique_block.0,
                unique_block.1,
            )
            .execute(&mut *transaction)
            .await
            .map_err(PushMetadataError::UnableToExpireBlocks)?;
        }
    }
    // Create background tasks for our storage hosts to notify them to prune blocks
    for (storage_host_id, prune_blocks) in prune_blocks_tasks_map {
        PruneBlocksTask::new(storage_host_id, prune_blocks)
            .enqueue::<banyan_task::SqliteTaskStore>(&mut *database)
            .await
            .map_err(PushMetadataError::UnableEnqueuePruneBlocksTask)?;
    }
    transaction
        .commit()
        .await
        .map_err(PushMetadataError::UnableToExpireBlocks)?;

    Ok(())
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

    #[error("corrupted uuid provided: {0}")]
    DatabaseUuidCorrupted(uuid::Error),

    #[error("failed to mark bucket key as approved: {0}")]
    KeyApprovalFailed(sqlx::Error),

    #[error("failed to create entry for metadata in the database: {0}")]
    MetadataRegistrationFailed(sqlx::Error),

    #[error("unable to parse valid boundary: {0}")]
    InvalidBoundary(multer::Error),

    #[error("provided request data couldn't be decoded: {0}")]
    InvalidRequestData(serde_json::Error),

    #[error("invalid CID provided: {0}")]
    InvalidCid(cid::Error),

    #[error("account reached upload quota and recording the failure may have failed: {0:?}")]
    LimitReached(Option<sqlx::Error>),

    #[error("request did not contain required data segment")]
    MissingRequestData,

    #[error("request did not contain required metadata segment")]
    MissingMetadata,

    #[error("unable to locate block with provided CID: {0}")]
    NoBlock(String),

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

    #[error("failed to create new storage authorization: {0}")]
    UnableToGenerateAuthorization(#[from] StorageAuthorizationError),

    #[error("unable to identify how much data user has stored with each storage provider")]
    UnableToIdentifyStoredAmount(sqlx::Error),

    #[error("couldn't locate existing storage authorizations for account: {0}")]
    UnableToRetrieveAuthorizations(sqlx::Error),

    #[error("couldn't mark blocks as expired: {0}")]
    UnableToExpireBlocks(sqlx::Error),

    #[error("couldn't enqueue a task to prune blocks")]
    UnableEnqueuePruneBlocksTask(banyan_task::TaskStoreError),

    #[error("failed to store metadata on disk: {0}, marking as failed might have had an error as well: {1:?}")]
    UploadStoreFailed(StoreMetadataError, Option<sqlx::Error>),
}

impl IntoResponse for PushMetadataError {
    fn into_response(self) -> Response {
        match &self {
            PushMetadataError::BrokenMultipartField(_)
            | PushMetadataError::InvalidBoundary(_)
            | PushMetadataError::InvalidRequestData(_)
            | PushMetadataError::InvalidCid(_)
            | PushMetadataError::NoBlock(_)
            | PushMetadataError::MissingRequestData => {
                let err_msg = serde_json::json!({"msg": "invalid request"});
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            PushMetadataError::LimitReached(_) => {
                let err_msg = serde_json::json!({"msg": "you have hit your account storage limit"});
                (StatusCode::PAYLOAD_TOO_LARGE, Json(err_msg)).into_response()
            }
            PushMetadataError::NoAvailableStorage => {
                tracing::error!("no storage host available with capacity to store pushed data!");
                let err_msg = serde_json::json!({"msg": "an internal server error occurred"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            PushMetadataError::NoAuthorizedBucket => {
                let err_msg = serde_json::json!({"msg": "not found"});
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            }
            _ => {
                tracing::error!("failed to push metadata: {self}");
                let err_msg = serde_json::json!({"msg": "an internal server error occurred"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}

#[derive(Deserialize)]
pub struct PushMetadataRequest {
    pub root_cid: String,
    pub metadata_cid: String,

    pub expected_data_size: i64,
    /// Fingerprints of Public Bucket Keys
    #[serde(rename = "valid_keys")]
    pub included_key_fingerprints: Vec<String>,

    pub deleted_block_cids: BTreeSet<String>,
}

#[derive(sqlx::FromRow)]
struct SelectedStorageHost {
    pub id: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug, thiserror::Error)]
pub enum StorageAuthorizationError {
    #[error("failed to record new grant storage authorization in the database: {0}")]
    GrantRecordingFailed(sqlx::Error),

    #[error("failed to sign new storage authorization: {0}")]
    SignatureFailed(jwt_simple::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum StoreMetadataError {
    #[error("failed to finalize storage to disk: {0}")]
    NotFinalized(std::io::Error),

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
