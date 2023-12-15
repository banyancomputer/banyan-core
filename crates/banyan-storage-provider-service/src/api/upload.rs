use std::path::PathBuf;

use axum::extract::{BodyStream, Json, State};
use axum::headers::{ContentLength, ContentType};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::TypedHeader;
use banyan_car_analyzer::{CarReport, StreamingCarAnalyzer, StreamingCarAnalyzerError};
use banyan_object_store::{ObjectStore, ObjectStoreError, ObjectStorePath};
use banyan_task::TaskLikeExt;
use futures::{TryStream, TryStreamExt};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncWrite, AsyncWriteExt};
use uuid::Uuid;

use crate::app::AppState;
use crate::database::{map_sqlx_error, Database, DatabaseError};
use crate::extractors::AuthenticatedClient;
use crate::tasks::ReportUploadTask;

/// Limit on the size of the JSON request that accompanies an upload.
const UPLOAD_REQUEST_SIZE_LIMIT: u64 = 100 * 1_024;

#[derive(Deserialize, Serialize)]
pub struct UploadRequest {
    metadata_id: Uuid,
    content_hash: String,
}

pub async fn handler(
    State(state): State<AppState>,
    client: AuthenticatedClient,
    store: ObjectStore,
    TypedHeader(content_len): TypedHeader<ContentLength>,
    TypedHeader(content_type): TypedHeader<ContentType>,
    body: BodyStream,
) -> Result<Response, UploadError> {
    let mut db = state.database();
    let reported_body_length = content_len.0;
    if reported_body_length > client.remaining_storage() {
        return Err(UploadError::InsufficientAuthorizedStorage(
            reported_body_length,
            client.remaining_storage(),
        ));
    }

    let mime_ct = mime::Mime::from(content_type);
    let boundary = multer::parse_boundary(mime_ct).unwrap();
    let constraints = multer::Constraints::new()
        .allowed_fields(vec!["request-data", "car-upload"])
        .size_limit(
            multer::SizeLimit::new()
                .for_field("request-data", UPLOAD_REQUEST_SIZE_LIMIT)
                .for_field("car-upload", client.remaining_storage()),
        );

    let mut multipart = multer::Multipart::with_constraints(body, boundary, constraints);

    let request_field = multipart
        .next_field()
        .await
        .map_err(UploadError::RequestFieldUnavailable)?
        .ok_or(UploadError::RequestFieldMissing)?;

    // TODO: validate name is request-data (request_data_field.name())
    // TODO: validate type is application/json (request_data_field.content_type())

    let request: UploadRequest = request_field
        .json()
        .await
        .map_err(UploadError::InvalidRequestData)?;
    let content_hash = request.content_hash;

    let (upload_id, tmp_file_path) =
        record_upload_beginning(&db, client.id(), request.metadata_id, reported_body_length)
            .await?;

    // todo: should make sure I have a clean up task that watches for failed uploads and handles
    // them appropriately

    let car_field = multipart
        .next_field()
        .await
        .map_err(UploadError::DataFieldUnavailable)?
        .ok_or(UploadError::DataFieldMissing)?;

    // TODO: validate name is car-upload (request_data_field.name())
    // TODO: validate type is "application/vnd.ipld.car; version=2" (request_data_field.content_type())

    let store_path = ObjectStorePath::from(tmp_file_path.as_str());
    let (multipart_resume_id, mut writer) = match store.put_multipart(&store_path).await {
        Ok(mp) => mp,
        Err(err) => {
            handle_failed_upload(&db, &upload_id).await;
            return Err(UploadError::StoreUnavailable(err));
        }
    };

    match process_upload_stream(
        &db,
        &upload_id,
        reported_body_length as usize,
        car_field,
        content_hash,
        &mut writer,
    )
    .await
    {
        Ok(cr) => {
            writer
                .shutdown()
                .await
                .map_err(UploadStreamError::WriteFailed)?;

            handle_successful_upload(&db, &store, &cr, &upload_id, &store_path).await?;
            ReportUploadTask::new(
                client.storage_grant_id(),
                request.metadata_id,
                cr.cids(),
                cr.total_size(),
            )
            .enqueue::<banyan_task::SqliteTaskStore>(&mut db)
            .await
            .map_err(UploadError::FailedToEnqueueTask)?;

            Ok((StatusCode::NO_CONTENT, ()).into_response())
        }
        Err(err) => {
            // todo: we don't care in the response if this fails, but if it does we will want to
            // clean it up in the future which should be handled by a background task
            let _ = store
                .abort_multipart(&store_path, &multipart_resume_id)
                .await;
            handle_failed_upload(&db, &upload_id).await;
            Err(err.into())
        }
    }
}

async fn handle_failed_upload(db: &Database, upload_id: &str) {
    // attempt to report the upload as failed, but that fails we'll need to handle it in a
    // future clean-up task. todo: should actually just enqueue and entire clean up process
    // and report this as failed there...
    let _ = record_upload_failed(db, upload_id).await;
}

async fn handle_successful_upload(
    db: &Database,
    _store: &ObjectStore,
    car_report: &CarReport,
    upload_id: &str,
    _file_path: &ObjectStorePath,
) -> Result<(), UploadError> {
    //let mut path_iter = file_path.parts();
    // discard the uploading/ prefix
    //let _ = path_iter.next();
    //let mut final_path: ObjectStorePath = path_iter.collect();

    //// todo: validate the local store doesn't use copy to handle this as some of the other stores
    //// do...
    //if let Err(err) = store.rename_if_not_exists(file_path, &final_path).await {
    //    // this is a weird error, its successfully written to our file store so we have it and can
    //    // serve it we just want to make sure we don't update the path in the DB and clean it up
    //    // later
    //    tracing::error!("unable to rename upload, leaving it in place: {err}");
    //    // todo: background a task to handle correcting this issue when it occurs
    //    final_path = file_path.clone();
    //}

    // todo: should definitely do a db transaction before the attempted rename, committing only if
    // the rename is successfuly
    sqlx::query(
        r#"
                    UPDATE uploads SET
                            state = 'complete',
                            final_size = $1,
                            integrity_hash  = $2
                        WHERE id = $3;
                "#,
    )
    .bind(car_report.total_size() as i64)
    .bind(car_report.integrity_hash())
    .bind(upload_id)
    .execute(db)
    .await
    .map_err(map_sqlx_error)?;

    Ok(())
}

async fn record_upload_beginning(
    db: &Database,
    client_id: Uuid,
    metadata_id: Uuid,
    reported_size: u64,
) -> Result<(String, String), UploadError> {
    let mut tmp_upload_path = PathBuf::new();

    tmp_upload_path.push(format!("{metadata_id}.car"));

    let tmp_upload_path = tmp_upload_path.display().to_string();

    let upload_id = sqlx::query_scalar(
        r#"
                    INSERT INTO
                        uploads (client_id, metadata_id, reported_size, file_path, state)
                        VALUES ($1, $2, $3, $4, 'started')
                        RETURNING id;
                "#,
    )
    .bind(client_id.to_string())
    .bind(metadata_id.to_string())
    .bind(reported_size as i64)
    .bind(&tmp_upload_path)
    .fetch_one(db)
    .await
    .map_err(map_sqlx_error)
    .map_err(UploadError::Database)?;
    Ok((upload_id, tmp_upload_path))
}

async fn record_upload_failed(db: &Database, upload_id: &str) -> Result<(), UploadError> {
    let _rows_affected: i32 = sqlx::query_scalar(
        r#"
                    UPDATE uploads SET state = 'failed' WHERE id = $1;
                "#,
    )
    .bind(upload_id)
    .fetch_one(db)
    .await
    .map_err(map_sqlx_error)?;
    Ok(())
}

async fn process_upload_stream<S>(
    db: &Database,

    upload_id: &str,
    expected_size: usize,
    mut stream: S,
    content_hash: String,
    writer: &mut Box<dyn AsyncWrite + Unpin + Send>,
) -> Result<CarReport, UploadStreamError>
where
    S: TryStream<Ok = bytes::Bytes, Error = multer::Error> + Unpin,
{
    let mut car_analyzer = StreamingCarAnalyzer::new();
    let mut warning_issued = false;
    let mut hasher = blake3::Hasher::new();
    while let Some(chunk) = stream
        .try_next()
        .await
        .map_err(UploadStreamError::ReadFailed)?
    {
        hasher.update(&chunk);
        car_analyzer.add_chunk(&chunk)?;
        writer
            .write_all(&chunk)
            .await
            .map_err(UploadStreamError::WriteFailed)?;

        while let Some(block_meta) = car_analyzer.next().await? {
            let cid_string = block_meta
                .cid()
                .to_string_of_base(cid::multibase::Base::Base64Url)
                .expect("parsed cid to unparse");

            sqlx::query(
                r#"
                            INSERT OR IGNORE INTO
                                blocks (cid, data_length)
                                VALUES ($1, $2);
                        "#,
            )
            .bind(cid_string.clone())
            .bind(block_meta.length() as i64)
            .execute(db)
            .await
            .map_err(map_sqlx_error)?;

            let block_id: Uuid = {
                let cid_id: String =
                    sqlx::query_scalar("SELECT id FROM blocks WHERE cid = $1 LIMIT 1;")
                        .bind(cid_string)
                        .fetch_one(db)
                        .await
                        .map_err(map_sqlx_error)?;

                Uuid::parse_str(&cid_id)
                    .map_err(|_| UploadStreamError::DatabaseCorruption("cid uuid parsing"))?
            };

            // create uploads_blocks row with the block information
            sqlx::query(
                r#"
                                INSERT INTO
                                    uploads_blocks (upload_id, block_id, byte_offset)
                                    VALUES ($1, $2, $3);
                            "#,
            )
            .bind(upload_id)
            .bind(block_id.to_string())
            .bind(block_meta.offset() as i64)
            .execute(db)
            .await
            .map_err(map_sqlx_error)?;
        }

        if car_analyzer.seen_bytes() as usize > expected_size && !warning_issued {
            warning_issued = true;
            tracing::warn!(
                "client is streaming more data than was claimed to be present in the upload"
            );
        }
    }
    let hash = hasher.finalize().to_string();
    if hash != content_hash {
        return Err(UploadStreamError::ParseError(
            StreamingCarAnalyzerError::MismatchedHash,
        ));
    }
    Ok(car_analyzer.report()?)
}

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

    #[error("account is not authorized to store {0} bytes, {1} bytes are still authorized")]
    InsufficientAuthorizedStorage(u64, u64),

    #[error("a CID from our internal reports wasn't convertable: {0}")]
    InvalidInternalCid(cid::Error),

    #[error("request's data payload was malformed")]
    InvalidRequestData(multer::Error),

    #[error("failed to acquire request field from body")]
    RequestFieldUnavailable(multer::Error),

    #[error("we expected a request field but received nothing")]
    RequestFieldMissing,

    #[error("unable to open store for properly authorized data upload: {0}")]
    StoreUnavailable(#[from] ObjectStoreError),

    #[error("streaming car upload failed")]
    StreamingFailed(#[from] UploadStreamError),
}

impl IntoResponse for UploadError {
    fn into_response(self) -> Response {
        use UploadError::*;

        match self {
            Database(_) | FailedToEnqueueTask(_) | InvalidInternalCid(_) | StoreUnavailable(_) => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            DataFieldUnavailable(_)
            | DataFieldMissing
            | InvalidRequestData(_)
            | RequestFieldUnavailable(_)
            | RequestFieldMissing => {
                let err_msg = serde_json::json!({ "msg": format!("{self}") });
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            InsufficientAuthorizedStorage(requested_bytes, remaining_bytes) => {
                tracing::warn!(upload_size = ?requested_bytes, remaining_storage = ?remaining_bytes, "user doesn't have sufficient storage capacity remaining");
                let err_msg = serde_json::json!({ "msg": format!("{self}") });
                (StatusCode::PAYLOAD_TOO_LARGE, Json(err_msg)).into_response()
            }
            StreamingFailed(stream_err) => stream_err.into_response(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UploadStreamError {
    #[error("encountered invalid data from the database for {0}")]
    DatabaseCorruption(&'static str),

    #[error("unable to record details about the stream in the database")]
    DatabaseFailure(#[from] DatabaseError),

    #[error("uploaded file was not a properly formatted car file")]
    ParseError(#[from] StreamingCarAnalyzerError),

    #[error("failed to read from client")]
    ReadFailed(multer::Error),

    #[error("failed to write to storage backend")]
    WriteFailed(std::io::Error),
}

impl IntoResponse for UploadStreamError {
    fn into_response(self) -> Response {
        use UploadStreamError::*;

        match self {
            DatabaseCorruption(indicator) => {
                tracing::error!("detected a corrupted reference in the database: {indicator}");
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            DatabaseFailure(err) => {
                tracing::error!("recording block details in db failed: {err}");
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            ParseError(err) => err.into_response(),
            ReadFailed(_) => {
                let err_msg = serde_json::json!({
                    "msg": format!("client stream went away before file upload was complete")
                });
                (StatusCode::UNPROCESSABLE_ENTITY, Json(err_msg)).into_response()
            }
            WriteFailed(err) => {
                tracing::error!("writing car file failed: {err}");
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
