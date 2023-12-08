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

use crate::app::{AppState, ServiceKey};
use crate::database::models::{Bucket, Metadata, MetadataState, NewMetadata};
use crate::database::{Database, DatabaseConnection};
use crate::extractors::{DataStore, UserIdentity};
use crate::tasks::{PruneBlock, PruneBlocksTask};
use crate::utils::car_buffer::CarBuffer;

/// The default quota we assume each storage host / staging service to provide (10 GiB limit until
/// payment is in place).
const ACCOUNT_STORAGE_QUOTA: i64 = 10 * 1_024 * 1_024 * 1_024;

/// Size limit of the pure metadata CAR file that is being uploaded (128MiB)
const CAR_DATA_SIZE_LIMIT: u64 = 128 * 1_024 * 1_024;

const CAR_MIME_TYPE: &'static mime::Mime =
    &mime::Mime::from_str("application/vnd.ipfs.car; version=2").expect("valid");

const ONE_HUNDRED_MIB: i64 = 100 * 1024 * 1024;

/// Upper size limit on the JSON payload that precedes a metadata CAR file upload (128KiB)
const REQUEST_DATA_SIZE_LIMIT: u64 = 128 * 1_024;

pub const STORAGE_TICKET_DURATION: u64 = 15 * 60; // 15 minutes

async fn bucket_change_in_progress(
    _conn: &DatabaseConnection,
    _bucket_id: &str,
) -> Result<bool, PushMetadataError> {
    Ok(false)
}

#[derive(Debug, thiserror::Error)]
pub enum PushMetadataRequestError {
    #[error("error occurred while querying database: {0}")]
    DbQueryFailure(#[from] sqlx::Error),

    #[error("the request was badly formatted: {0}")]
    InvalidMultipart(#[from] multer::Error),
}

pub async fn handler(
    user_id: UserIdentity,
    State(state): State<AppState>,
    store: DataStore,
    Path(bucket_id): Path<Uuid>,
    TypedHeader(content_type): TypedHeader<ContentType>,
    body: BodyStream,
) -> Result<Response, PushMetadataRequestError> {
    let bucket_id = bucket_id.to_string();
    let user_id = user_id.id().to_string();

    let span = tracing::info_span!("push_metadata_handler", bucket_id, user_id);
    let _guard = span.enter();

    let database = state.database();
    let service_key = state.secrets().service_key();

    let mut conn = database.begin().await?;

    if !Bucket::is_owned_by_user_id(&mut conn, &bucket_id, &user_id).await? {
        let err_msg = serde_json::json!({"msg": "not found"});
        return Ok((StatusCode::NOT_FOUND, Json(err_msg)).into_response());
    }

    if Bucket::change_in_progress(&mut conn, &bucket_id).await? {
        tracing::warn!("attempted upload to bucket while other write was in progress");
        let err_msg = serde_json::json!({"msg": "waiting for other upload to complete"});
        return Ok((StatusCode::CONFLICT, Json(err_msg)).into_response());
    }

    // Request is authorized, and we're ready to receive it. Start processing the multipart
    // payload...

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

    let data_field = match multipart.next_field().await? {
        Some(d) => d,
        None => {
            tracing::warn!("upload contained no data");
            let err_msg = serde_json::json!({"msg": "missing request data"});
            return Ok((StatusCode::BAD_REQUEST, Json(err_msg)).into_response());
        }
    };

    if !validate_field(&data_field, "car-upload", CAR_MIME_TYPE) {
        let err_msg = serde_json::json!({"msg": "upload data is unexpected type"});
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

    let fingerprints = request_data
            .included_key_fingerprints
            .iter()
            .map(String::as_str);

    Bucket::approve_keys_by_fingerprint(&mut conn, &bucket_id, fingerprints).await?;

    expire_deleted_blocks(
        &mut conn,
        &user_id.id(),
        &bucket_id,
        &request_data.deleted_block_cids,
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

    // Checkpoint the upload to the database so we can track failures, and perform any necessary
    // clean up behind the scenes.
    conn.commit().await?;

    let file_name = format!("{bucket_id}/{metadata_id}.car");
    let (hash, size) = store_metadata_stream(&store, file_name.as_str(), data_field).await?;

    let mut conn = database.begin().await?;
    Metadata::upload_complete(&mut conn, &metadata_id, &hash, size).await?;

    let expected_total_storage = currently_consumed_storage(&mut conn, &user_id)
        .await
        .map_err(PushMetadataError::UnableToCheckAccounting)?
        as i64;

    if expected_total_storage > ACCOUNT_STORAGE_QUOTA {
        tracing::warn!(?expected_total_storage, ?user_id, "account reached storage limit");
        let err_msg = serde_json::json!({"msg": "account reached available storage threshold"});
        return Ok((StatusCode::PAYLOAD_TOO_LARGE, Json(err_msg)).into_response());
    }

    if request_data.expected_data_size == 0 {
        mark_current(&mut conn, &metadata_id)
            .await
            .map_err(PushMetadataError::ActivationFailed)?;

        let resp_msg = serde_json::json!({"id": metadata_id, "state": "current"});
        return Ok((StatusCode::OK, Json(resp_msg)).into_response());
    }

    let storage_host = select_storage_host(&mut conn, request_data.expected_data_size).await?;

    let current_authorized_amount = existing_authorization(&mut conn, &user_id, &storage_host.id)
        .await
        .map_err(PushMetadataError::UnableToRetrieveAuthorizations)?;
    let current_stored_amount = currently_stored_at_provider(&mut conn, &user_id, &storage_host.id)
        .await
        .map_err(PushMetadataError::UnableToIdentifyStoredAmount)?;

    let mut storage_authorization: Option<String> = None;
    if (current_authorized_amount - current_stored_amount) < request_data.expected_data_size {
        let currently_required_amount = current_stored_amount + request_data.expected_data_size;
        let data_size_rounded =
            ((currently_required_amount / ONE_HUNDRED_MIB) + 1) * ONE_HUNDRED_MIB;

        let new_authorization = generate_new_storage_authorization(
            &database,
            &service_key,
            &user_id.id(),
            &storage_host,
            data_size_rounded,
        )
        .await
        .map_err(PushMetadataError::UnableToGenerateAuthorization)?;

        storage_authorization = Some(new_authorization);
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

async fn currently_consumed_storage(
    database: &Database,
    user_id: &str,
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
            b.user_id = $1 AND m.state IN ('current', 'outdated', 'pending');"#,
        user_id,
    )
    .fetch_optional(database)
    .await?;

    Ok(maybe_stored.unwrap_or(0))
}

async fn currently_stored_at_provider(
    database: &Database,
    user_id: &str,
    storage_host_id: &str,
) -> Result<i64, sqlx::Error> {
    let res: Result<Option<i64>, _> = sqlx::query_scalar!(
        r#"SELECT SUM(m.data_size) as total_data_size FROM metadata m
               JOIN storage_hosts_metadatas_storage_grants shmg ON shmg.metadata_id = m.id
               JOIN storage_grants sg ON shmg.storage_grant_id = sg.id
               WHERE sg.user_id = $1 AND shmg.storage_host_id = $2;"#,
        user_id,
        storage_host_id,
    )
    .fetch_one(database)
    .await;

    res.map(|amt| amt.unwrap_or(0))
}

async fn existing_authorization(
    database: &Database,
    user_id: &str,
    storage_host_id: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT authorized_amount FROM storage_grants
               WHERE user_id = $1
                   AND storage_host_id = $2
                   AND redeemed_at IS NOT NULL
               ORDER BY created_at DESC
               LIMIT 1;"#,
        user_id,
        storage_host_id,
    )
    .fetch_optional(database)
    .await
    .map(|amt| amt.unwrap_or(0))
}

#[derive(Debug, Serialize, Deserialize)]
struct Capabilities {
    #[serde(rename = "cap")]
    capabilities: serde_json::Map<String, serde_json::Value>,
}

async fn generate_new_storage_authorization(
    database: &Database,
    service_key: &ServiceKey,
    user_identity: &UserIdentity,
    storage_host: &SelectedStorageHost,
    authorized_amount: i64,
) -> Result<String, StorageAuthorizationError> {
    let user_id = user_identity.id().to_string();
    let key_fingerprint = user_identity.key_fingerprint().to_string();
    let storage_grant_id = sqlx::query_scalar!(
        r#"INSERT INTO storage_grants (storage_host_id, user_id, authorized_amount)
            VALUES ($1, $2, $3)
            RETURNING id;"#,
        storage_host.id,
        user_id,
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
    // TODO: this should be configurable
    .with_issuer("banyan-platform")
    .with_subject(format!("{}@{}", user_id, key_fingerprint))
    .invalid_before(Clock::now_since_epoch() - Duration::from_secs(30));

    claims.create_nonce();
    claims.issued_at = Some(Clock::now_since_epoch());

    let bearer_token = service_key
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
    normalized_cid: String,
    metadata_id: String,
    storage_host_id: String,
}

async fn expire_deleted_blocks(
    database: &Database,
    user_id: &Uuid,
    bucket_id: &Uuid,
    deleted_block_cids: &BTreeSet<String>,
) -> Result<(), PushMetadataError> {
    let user_id = user_id.to_string();
    let bucket_id = bucket_id.to_string();
    let mut prune_blocks_tasks_map: HashMap<Uuid, Vec<PruneBlock>> = HashMap::new();

    // Check if block set is empty
    if deleted_block_cids.is_empty() {
        return Ok(());
    }

    // Build a query to identify the blocks that need to get expired
    let mut builder = sqlx::QueryBuilder::new(
        r#"SELECT blocks.id AS block_id, blocks.cid AS normalized_cid, m.id AS metadata_id, sh.id AS storage_host_id
            FROM block_locations AS bl
            JOIN blocks ON blocks.id = bl.block_id
            JOIN storage_hosts AS sh ON sh.id = bl.storage_host_id
            JOIN metadata AS m ON m.id = bl.metadata_id
            JOIN buckets AS b ON b.id = m.bucket_id
            WHERE b.user_id ="#,
    );
    builder.push_bind(user_id);
    builder.push(" AND b.id = ");
    builder.push_bind(bucket_id);
    builder.push(" AND blocks.cid IN (");
    for (i, cid) in deleted_block_cids.iter().enumerate() {
        let normalized_cid = Cid::from_str(cid)
            .map_err(PushMetadataError::InvalidCid)?
            .to_string_of_base(Base::Base64Url)
            .map_err(PushMetadataError::InvalidCid)?;

        if i > 0 {
            builder.push(", ");
        }

        builder.push_bind(normalized_cid);
    }
    builder.push(")");

    // Execute the query
    let query = builder.build_query_as::<UniqueBlockLocation>();
    let unique_block_locations = query
        .fetch_all(database)
        .await
        .map_err(PushMetadataError::UnableToExpireBlocks)?;

    // If no blocks were found, we can stop here.
    if unique_block_locations.is_empty() {
        return Ok(());
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
            normalized_cid: unique_block_location.normalized_cid.clone(),
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

    // Update the unique blocks that need to get expired
    let mut builder = sqlx::query_builder::QueryBuilder::new(
        r#"UPDATE block_locations
        SET expired_at = CURRENT_TIMESTAMP
        WHERE (block_id, metadata_id) IN ("#,
    );
    for (i, (block_id, metadata_id)) in unique_blocks.iter().enumerate() {
        if i > 0 {
            builder.push(", ");
        }
        builder.push("(");
        builder.push_bind(block_id);
        builder.push(", ");
        builder.push_bind(metadata_id);
        builder.push(")");
    }
    builder.push(")");

    // Begin a transaction for updating block and task state
    let mut transaction = database
        .begin()
        .await
        .map_err(PushMetadataError::UnableToExpireBlocks)?;

    // Execute the query to update blocks
    let query = builder.build();
    query
        .execute(&mut *transaction)
        .await
        .map_err(PushMetadataError::UnableToExpireBlocks)?;

    // Create background tasks for our storage hosts to notify them to prune blocks
    for (storage_host_id, prune_blocks) in prune_blocks_tasks_map {
        PruneBlocksTask::new(storage_host_id, prune_blocks)
            .enqueue_with_connection::<banyan_task::SqliteTaskStore>(&mut transaction)
            .await
            .map_err(PushMetadataError::UnableEnqueuePruneBlocksTask)?;

        Queue::name("default")
            .metrics::<banyan_task::SqliteTaskStore>(&mut transaction)
            .await?;
    }

    // Commit the txn
    transaction
        .commit()
        .await
        .map_err(PushMetadataError::UnableToExpireBlocks)?;

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum PushMetadataError {
    #[error("failed to run query: {0}")]
    QueryFailure(#[from] sqlx::Error),
}

//#[derive(Debug, thiserror::Error)]
//pub enum PushMetadataError {
//
//    #[error("failed updating zero data metadata to current")]
//    ActivationFailed(sqlx::Error),
//
//    #[error("failed to validate bucket was authorized by user: {0}")]
//    BucketAuthorizationFailed(sqlx::Error),
//
//    #[error("unable to pull next multipart field: {0}")]
//    BrokenMultipartField(multer::Error),
//
//    #[error("failed to record metadata size and hash: {0}")]
//    DataMetaStoreFailed(sqlx::Error),
//
//    #[error("corrupted uuid provided: {0}")]
//    DatabaseUuidCorrupted(uuid::Error),
//
//    #[error("failed to mark bucket key as approved: {0}")]
//    KeyApprovalFailed(sqlx::Error),
//
//    #[error("failed to create entry for metadata in the database: {0}")]
//    MetadataRegistrationFailed(sqlx::Error),
//
//    #[error("unable to parse valid boundary: {0}")]
//    InvalidBoundary(multer::Error),
//
//    #[error("provided request data couldn't be decoded: {0}")]
//    InvalidRequestData(serde_json::Error),
//
//    #[error("invalid CID provided: {0}")]
//    InvalidCid(cid::Error),
//
//    #[error("account reached upload quota and recording the failure may have failed: {0:?}")]
//    LimitReached(Option<sqlx::Error>),
//
//    #[error("request did not contain required data segment")]
//    MissingRequestData,
//
//    #[error("request did not contain required metadata segment")]
//    MissingMetadata,
//
//    #[error("unable to locate a bucket for the current authorized user")]
//    NoAuthorizedBucket,
//
//    #[error("no storage host is available with sufficient storage")]
//    NoAvailableStorage,
//
//    #[error("unable to retrieve request data: {0}")]
//    RequestDataUnavailable(multer::Error),
//
//    #[error("failed to query for available storage host: {0}")]
//    StorageHostLookupFailed(sqlx::Error),
//
//    #[error("unable to determine if user is within their quota: {0}")]
//    UnableToCheckAccounting(sqlx::Error),
//
//    #[error("failed to create new storage authorization: {0}")]
//    UnableToGenerateAuthorization(#[from] StorageAuthorizationError),
//
//    #[error("unable to identify how much data user has stored with each storage provider")]
//    UnableToIdentifyStoredAmount(sqlx::Error),
//
//    #[error("unable to locate existing storage authorizations for account: {0}")]
//    UnableToRetrieveAuthorizations(sqlx::Error),
//
//    #[error("unable to mark blocks as expired: {0}")]
//    UnableToExpireBlocks(sqlx::Error),
//
//    #[error("unable to enqueue a task to prune blocks {0}")]
//    UnableEnqueuePruneBlocksTask(banyan_task::TaskStoreError),
//
//    #[error("failed to store metadata on disk: {0}, marking as failed might have had an error as well: {1:?}")]
//    UploadStoreFailed(StoreMetadataError, Option<sqlx::Error>),
//}
//
//impl IntoResponse for PushMetadataError {
//    fn into_response(self) -> Response {
//        match &self {
//            PushMetadataError::ChangeInProgress => {
//                let err_msg = serde_json::json!({"msg": "metadata write already in progress!"});
//                (StatusCode::CONFLICT, Json(err_msg)).into_response()
//            }
//            PushMetadataError::BrokenMultipartField(_)
//            | PushMetadataError::InvalidBoundary(_)
//            | PushMetadataError::InvalidRequestData(_)
//            | PushMetadataError::InvalidCid(_)
//            | PushMetadataError::MissingRequestData => {
//                let err_msg = serde_json::json!({"msg": "invalid request"});
//                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
//            }
//            PushMetadataError::NoAvailableStorage => {
//                tracing::error!("no storage host available with capacity to store pushed data!");
//                let err_msg = serde_json::json!({"msg": "an internal server error occurred"});
//                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
//            }
//            PushMetadataError::NoAuthorizedBucket => {
//                let err_msg = serde_json::json!({"msg": "not found"});
//                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
//            }
//            _ => {
//                tracing::error!("failed to push metadata: {self}");
//                let err_msg = serde_json::json!({"msg": "an internal server error occurred"});
//                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
//            }
//        }
//    }
//}

#[derive(Deserialize)]
pub struct PushMetadataRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_metadata_cid: Option<String>,

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

#[cfg(test)]
mod tests {
    use super::*;

    // Single User
    const USER_ID: &str = "00000000-0000-0000-0000-000000000000";

    // Two Buckets
    const BUCKET_ID_1: &str = "00000000-0000-0000-0000-000000000000";
    const BUCKET_ID_2: &str = "00000000-0000-0000-0000-000000000001";

    // Two Storage Hosts
    const STORAGE_HOST_ID_1: &str = "00000000-0000-0000-0000-000000000000";
    const STORAGE_HOST_ID_2: &str = "00000000-0000-0000-0000-000000000001";

    // Three pieces of metadata
    // Two under Bucket 1
    const METADATA_ID_1: &str = "00000000-0000-0000-0000-000000000000";
    const METADATA_ID_2: &str = "00000000-0000-0000-0000-000000000001";
    // One under Bucket 2
    const METADATA_ID_3: &str = "00000000-0000-0000-0000-000000000002";

    // 4 total blocks with random CIDs
    const BLOCK_ID_1: &str = "00000000-0000-0000-0000-000000000000";
    const BLOCK_ID_2: &str = "00000000-0000-0000-0000-000000000001";
    const BLOCK_ID_3: &str = "00000000-0000-0000-0000-000000000002";
    const BLOCK_ID_4: &str = "00000000-0000-0000-0000-000000000003";
    const BLOCK_CID_1: &str = "bafkreicvwpjh72yeufe5dtsxytdgsznckjxqaqinfe7wzjv3cb25sxy23u";
    const BLOCK_CID_2: &str = "bafkreif3ayrfp6qlqhn2nqfkjt7kjz7inydzuclgkxcpdegn2o7gtqirga";
    const BLOCK_CID_3: &str = "bafkreidwi6m7kyz3l2qlltxwvyv2idgzc7gsgqpfgnllq5m22ylwccrrsu";
    const BLOCK_CID_4: &str = "bafkreiainfete3i5wkr4aia3jtw263j53h3gwj6weuqldwluqr4kdtet5y";

    // 6 example block locations over 3 metadatas, 4 blocks, and 2 storage hosts
    const BLOCK_LOCATION_1: (&str, &str, &str) = (METADATA_ID_1, BLOCK_ID_1, STORAGE_HOST_ID_1);
    const BLOCK_LOCATION_2: (&str, &str, &str) = (METADATA_ID_1, BLOCK_ID_2, STORAGE_HOST_ID_1);
    const BLOCK_LOCATION_3: (&str, &str, &str) = (METADATA_ID_2, BLOCK_ID_2, STORAGE_HOST_ID_1);
    const BLOCK_LOCATION_4: (&str, &str, &str) = (METADATA_ID_2, BLOCK_ID_3, STORAGE_HOST_ID_2);
    const BLOCK_LOCATION_5: (&str, &str, &str) = (METADATA_ID_3, BLOCK_ID_3, STORAGE_HOST_ID_2);
    const BLOCK_LOCATION_6: (&str, &str, &str) = (METADATA_ID_3, BLOCK_ID_4, STORAGE_HOST_ID_2);

    #[tokio::test]
    async fn expire_bucket_1_blocks() {
        let db_conn = setup_expired_blocks_test().await;
        let user_id = Uuid::parse_str(USER_ID).expect("user_id");
        let bucket_id = Uuid::parse_str(BUCKET_ID_1).expect("bucket_id");
        let mut deleted_blocks = BTreeSet::new();
        deleted_blocks.insert(BLOCK_CID_1.to_string());
        deleted_blocks.insert(BLOCK_CID_2.to_string());
        deleted_blocks.insert(BLOCK_CID_3.to_string());
        deleted_blocks.insert(BLOCK_CID_4.to_string());

        expire_deleted_blocks(&db_conn, &user_id, &bucket_id, &deleted_blocks)
            .await
            .expect("success");

        expect_all_expired(
            &db_conn,
            &[
                BLOCK_LOCATION_1,
                BLOCK_LOCATION_2,
                BLOCK_LOCATION_3,
                BLOCK_LOCATION_4,
            ],
        )
        .await;
        expect_none_expired(&db_conn, &[BLOCK_LOCATION_5, BLOCK_LOCATION_6]).await;
    }

    #[tokio::test]
    async fn expire_bucket_2_blocks() {
        let db_conn = setup_expired_blocks_test().await;
        let user_id = Uuid::parse_str(USER_ID).expect("user_id");
        let bucket_id = Uuid::parse_str(BUCKET_ID_2).expect("bucket_id");
        let mut deleted_blocks = BTreeSet::new();
        deleted_blocks.insert(BLOCK_CID_1.to_string());
        deleted_blocks.insert(BLOCK_CID_2.to_string());
        deleted_blocks.insert(BLOCK_CID_3.to_string());
        deleted_blocks.insert(BLOCK_CID_4.to_string());

        expire_deleted_blocks(&db_conn, &user_id, &bucket_id, &deleted_blocks)
            .await
            .expect("success");

        expect_all_expired(&db_conn, &[BLOCK_LOCATION_5, BLOCK_LOCATION_6]).await;
        expect_none_expired(
            &db_conn,
            &[
                BLOCK_LOCATION_1,
                BLOCK_LOCATION_2,
                BLOCK_LOCATION_3,
                BLOCK_LOCATION_4,
            ],
        )
        .await;
    }

    #[tokio::test]
    async fn expire_no_blocks() {
        let db_conn = setup_expired_blocks_test().await;
        let user_id = Uuid::parse_str(USER_ID).expect("user_id");
        let bucket_id = Uuid::parse_str(BUCKET_ID_1).expect("bucket_id");
        let deleted_blocks = BTreeSet::new();

        expire_deleted_blocks(&db_conn, &user_id, &bucket_id, &deleted_blocks)
            .await
            .expect("success");

        expect_none_expired(
            &db_conn,
            &[
                BLOCK_LOCATION_1,
                BLOCK_LOCATION_2,
                BLOCK_LOCATION_3,
                BLOCK_LOCATION_4,
                BLOCK_LOCATION_5,
                BLOCK_LOCATION_6,
            ],
        )
        .await;
    }

    #[tokio::test]
    async fn expire_na_block() {
        let db_conn = setup_expired_blocks_test().await;
        let user_id = Uuid::parse_str(USER_ID).expect("user_id");
        let bucket_id = Uuid::parse_str(BUCKET_ID_1).expect("bucket_id");
        let mut deleted_blocks = BTreeSet::new();
        // Specify an unknown block
        deleted_blocks
            .insert("bafkreidz7iubrzo2vmzns47oqjqre3yzts3mzjuk4nciouhvljv2axxynm".to_string());

        expire_deleted_blocks(&db_conn, &user_id, &bucket_id, &deleted_blocks)
            .await
            .expect("success");

        expect_none_expired(
            &db_conn,
            &[
                BLOCK_LOCATION_1,
                BLOCK_LOCATION_2,
                BLOCK_LOCATION_3,
                BLOCK_LOCATION_4,
                BLOCK_LOCATION_5,
                BLOCK_LOCATION_6,
            ],
        )
        .await;
    }

    async fn expect_all_expired(db_conn: &sqlx::SqlitePool, locs: &[(&str, &str, &str)]) {
        let mut builder = sqlx::QueryBuilder::new(
            r#"SELECT * FROM block_locations WHERE expired_at = NULL AND (metadata_id, block_id, storage_host_id) IN ("#,
        );

        for (i, (metadata_id, block_id, storage_host_id)) in locs.iter().enumerate() {
            if i > 0 {
                builder.push(", ");
            }
            builder.push("(");
            builder.push_bind(metadata_id);
            builder.push(", ");
            builder.push_bind(block_id);
            builder.push(", ");
            builder.push_bind(storage_host_id);
            builder.push(")");
        }
        builder.push(")");

        // Execute the query
        let query = builder.build();
        let rows = query.fetch_all(db_conn).await.expect("db operation");
        assert!(rows.is_empty());
    }

    async fn expect_none_expired(db_conn: &sqlx::SqlitePool, locs: &[(&str, &str, &str)]) {
        let mut builder = sqlx::QueryBuilder::new(
            r#"SELECT * FROM block_locations WHERE expired_at != NULL AND (metadata_id, block_id, storage_host_id) IN ("#,
        );

        for (i, (metadata_id, block_id, storage_host_id)) in locs.iter().enumerate() {
            if i > 0 {
                builder.push(", ");
            }
            builder.push("(");
            builder.push_bind(metadata_id);
            builder.push(", ");
            builder.push_bind(block_id);
            builder.push(", ");
            builder.push_bind(storage_host_id);
            builder.push(")");
        }
        builder.push(")");

        // Execute the query
        let query = builder.build();
        let rows = query.fetch_all(db_conn).await.expect("db operation");
        assert!(rows.is_empty());
    }

    async fn setup_expired_blocks_test() -> sqlx::SqlitePool {
        let db_conn = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("db setup");
        sqlx::migrate!("./migrations")
            .run(&db_conn)
            .await
            .expect("db setup");

        // Create a fake user
        sqlx::query!(
            r#"INSERT INTO users (id, email, display_name)
            VALUES ($1, $2, $3)"#,
            USER_ID,
            "user@email.com",
            "test user"
        )
        .execute(&db_conn)
        .await
        .expect("db setup");

        // Create fake storage hosts
        sqlx::query!(
            r#"INSERT INTO storage_hosts (id, name, url, fingerprint, pem, used_storage, available_storage)
            VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
            STORAGE_HOST_ID_1,
            "storage_host_1",
            "fingerprint_1",
            "pem_1",
            "hello.com",
            0,
            0
        )
        .execute(&db_conn)
        .await
        .expect("db setup");
        sqlx::query!(
            r#"INSERT INTO storage_hosts (id, name, url, fingerprint, pem, used_storage, available_storage)
            VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
            STORAGE_HOST_ID_2,
            "storage_host_2",
            "fingerprint_2",
            "pem_2",
            "hello.com",
            0,
            0
        )
        .execute(&db_conn)
        .await
        .expect("db setup");

        // Create fake buckets
        sqlx::query!(
            r#"INSERT INTO buckets (id, user_id, name, type, storage_class)
            VALUES ($1, $2, $3, $4, $5)"#,
            BUCKET_ID_1,
            USER_ID,
            "test_1",
            "test",
            "test"
        )
        .execute(&db_conn)
        .await
        .expect("db setup");
        sqlx::query!(
            r#"INSERT INTO buckets (id, user_id, name, type, storage_class)
            VALUES ($1, $2, $3, $4, $5)"#,
            BUCKET_ID_2,
            USER_ID,
            "test_2",
            "test",
            "test"
        )
        .execute(&db_conn)
        .await
        .expect("db setup");

        // Create fake metadata
        sqlx::query!(
            r#"INSERT into metadata (id, bucket_id, root_cid, metadata_cid, expected_data_size, state)
            VALUES ($1, $2, $3, $4, $5, $6);"#,
            METADATA_ID_1,
            BUCKET_ID_1,
            "doop",
            "doop",
            0,
            "state"
        )
        .execute(&db_conn)
        .await
        .expect("db setup");
        sqlx::query!(
            r#"INSERT into metadata (id, bucket_id, root_cid, metadata_cid, expected_data_size, state)
            VALUES ($1, $2, $3, $4, $5, $6);"#,
            METADATA_ID_2,
            BUCKET_ID_1,
            "doop",
            "doop",
            0,
            "state"
        )
        .execute(&db_conn)
        .await
        .expect("db setup");
        sqlx::query!(
            r#"INSERT into metadata (id, bucket_id, root_cid, metadata_cid, expected_data_size, state)
            VALUES ($1, $2, $3, $4, $5, $6);"#,
            METADATA_ID_3,
            BUCKET_ID_2,
            "doop",
            "doop",
            0,
            "state"
        )
        .execute(&db_conn)
        .await
        .expect("db setup");

        // Create fake blocks
        let normalized_block_cid = Cid::from_str(BLOCK_CID_1)
            .expect("valid cid")
            .to_string_of_base(Base::Base64Url)
            .expect("b64 cid");
        sqlx::query!(
            r#"INSERT INTO blocks (id, cid)
            VALUES ($1, $2);"#,
            BLOCK_ID_1,
            normalized_block_cid
        )
        .execute(&db_conn)
        .await
        .expect("db setup");
        let normalized_block_cid = Cid::from_str(BLOCK_CID_2)
            .expect("valid cid")
            .to_string_of_base(Base::Base64Url)
            .expect("b64 cid");
        sqlx::query!(
            r#"INSERT INTO blocks (id, cid)
            VALUES ($1, $2);"#,
            BLOCK_ID_2,
            normalized_block_cid
        )
        .execute(&db_conn)
        .await
        .expect("db setup");
        let normalized_block_cid = Cid::from_str(BLOCK_CID_3)
            .expect("valid cid")
            .to_string_of_base(Base::Base64Url)
            .expect("b64 cid");
        sqlx::query!(
            r#"INSERT INTO blocks (id, cid)
            VALUES ($1, $2);"#,
            BLOCK_ID_3,
            normalized_block_cid
        )
        .execute(&db_conn)
        .await
        .expect("db setup");
        let normalized_block_cid = Cid::from_str(BLOCK_CID_4)
            .expect("valid cid")
            .to_string_of_base(Base::Base64Url)
            .expect("b64 cid");
        sqlx::query!(
            r#"INSERT INTO blocks (id, cid)
            VALUES ($1, $2);"#,
            BLOCK_ID_4,
            normalized_block_cid
        )
        .execute(&db_conn)
        .await
        .expect("db setup");

        // Create fake block locations
        sqlx::query!(
            r#"INSERT INTO block_locations (metadata_id, block_id, storage_host_id)
            VALUES ($1, $2, $3);"#,
            BLOCK_LOCATION_1.0,
            BLOCK_LOCATION_1.1,
            BLOCK_LOCATION_1.2,
        )
        .execute(&db_conn)
        .await
        .expect("db setup");
        sqlx::query!(
            r#"INSERT INTO block_locations (metadata_id, block_id, storage_host_id)
            VALUES ($1, $2, $3);"#,
            BLOCK_LOCATION_2.0,
            BLOCK_LOCATION_2.1,
            BLOCK_LOCATION_2.2,
        )
        .execute(&db_conn)
        .await
        .expect("db setup");
        sqlx::query!(
            r#"INSERT INTO block_locations (metadata_id, block_id, storage_host_id)
            VALUES ($1, $2, $3);"#,
            BLOCK_LOCATION_3.0,
            BLOCK_LOCATION_3.1,
            BLOCK_LOCATION_3.2,
        )
        .execute(&db_conn)
        .await
        .expect("db setup");
        sqlx::query!(
            r#"INSERT INTO block_locations (metadata_id, block_id, storage_host_id)
            VALUES ($1, $2, $3);"#,
            BLOCK_LOCATION_4.0,
            BLOCK_LOCATION_4.1,
            BLOCK_LOCATION_4.2,
        )
        .execute(&db_conn)
        .await
        .expect("db setup");
        sqlx::query!(
            r#"INSERT INTO block_locations (metadata_id, block_id, storage_host_id)
            VALUES ($1, $2, $3);"#,
            BLOCK_LOCATION_5.0,
            BLOCK_LOCATION_5.1,
            BLOCK_LOCATION_5.2,
        )
        .execute(&db_conn)
        .await
        .expect("db setup");
        sqlx::query!(
            r#"INSERT INTO block_locations (metadata_id, block_id, storage_host_id)
            VALUES ($1, $2, $3);"#,
            BLOCK_LOCATION_6.0,
            BLOCK_LOCATION_6.1,
            BLOCK_LOCATION_6.2,
        )
        .execute(&db_conn)
        .await
        .expect("db setup");

        db_conn
    }
}
