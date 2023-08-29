use std::path::PathBuf;

use axum::extract::{BodyStream, Json};
use axum::headers::{ContentLength, ContentType};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::TypedHeader;
use futures::{TryFutureExt, TryStream, TryStreamExt};
use object_store::ObjectStore;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncWrite, AsyncWriteExt};
use uuid::Uuid;

use crate::database::{BareId, DbError, Executor};
use crate::extractors::{AuthenticatedClient, Database, UploadStore};

/// Limit on the size of the JSON request that accompanies an upload.
const UPLOAD_REQUEST_SIZE_LIMIT: u64 = 100 * 1_024;

#[derive(Deserialize, Serialize)]
pub struct UploadRequest {
    metadata_id: Uuid,
}

pub async fn handler(
    db: Database,
    client: AuthenticatedClient,
    store: UploadStore,
    TypedHeader(content_len): TypedHeader<ContentLength>,
    TypedHeader(content_type): TypedHeader<ContentType>,
    body: BodyStream,
) -> Result<Response, UploadError> {
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
                .for_field("car-upload", client.remaining_storage() as u64),
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

    let (upload_id, tmp_file_path) = record_upload_beginning(&db, client.id(), request.metadata_id, reported_body_length).await?;

    // todo: should make sure I have a clean up task that watches for failed uploads and handles
    // them appropriately

    let car_field = multipart
        .next_field()
        .await
        .map_err(UploadError::DataFieldUnavailable)?
        .ok_or(UploadError::DataFieldMissing)?;

    // TODO: validate name is car-upload (request_data_field.name())
    // TODO: validate type is "application/vnd.ipld.car; version=2" (request_data_field.content_type())

    let store_path = object_store::path::Path::from(tmp_file_path.as_str());
    let (_multipart_resume_id, mut writer) = match store.put_multipart(&store_path).await {
        Ok(mp) => mp,
        Err(err) => {
            handle_failed_upload(&db, upload_id, &tmp_file_path).await;
            return Err(UploadError::StoreUnavailable(err));
        }
    };

    //match process_upload_stream(car_field, &mut writer).await {
    //    Ok(sr) => {
    //        //writer.shutdown().await.map_err(
    //    }
    //}

    // handle upload
    //      * if it goes over content length produce a warning

    handle_successful_upload(&db, &store, upload_id, &tmp_file_path).await?;

    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

async fn handle_failed_upload(db: &Database, upload_id: Uuid, _file_path: &str) {
    // attempt to report the upload as failed, but that fails we'll need to handle it in a
    // future clean-up task. todo: should actually just enqueue and entire clean up process
    // and report this as failed there...
    let _ = record_upload_failed(&db, upload_id).await;

    // todo: should try and clean up the file if it was created
}

async fn handle_successful_upload(_db: &Database, _store: &UploadStore, _upload_id: Uuid, _file_path: &str) -> Result<(), UploadError> {
    // todo: move the file to its final resting place
    // todo: mark upload as successful
    // todo: should enqueue background task to notify the platform

    Ok(())
}

async fn record_upload_beginning(db: &Database, client_id: Uuid, metadata_id: Uuid, reported_size: u64) -> Result<(Uuid, String), UploadError> {
    let mut tmp_upload_path = PathBuf::new();

    tmp_upload_path.push("uploading");
    tmp_upload_path.push(client_id.to_string());
    tmp_upload_path.push(format!("{metadata_id}.car"));

    let tmp_upload_path = tmp_upload_path.display().to_string();

    let upload_id: Uuid = match db.ex() {
        #[cfg(feature = "postgres")]
        Executor::Postgres(ref mut conn) => {
            use crate::database::postgres;

            let bare_upload_id: BareId = sqlx::query_as(r#"
                    INSERT INTO
                        uploads (client_id, metadata_id, reported_size, file_path, state)
                        VALUES ($1, $2, $3, $4, 'started')
                        RETURNING id;
                "#)
                .bind(client_id.to_string())
                .bind(metadata_id.to_string())
                .bind(reported_size as i64)
                .bind(&tmp_upload_path)
                .fetch_one(conn)
                .await
                .map_err(postgres::map_sqlx_error)
                .map_err(UploadError::Database)?;

            Uuid::parse_str(&bare_upload_id.id).unwrap()
        }

        #[cfg(feature = "sqlite")]
        Executor::Sqlite(ref mut conn) => {
            use crate::database::sqlite;

            let bare_upload_id: BareId = sqlx::query_as(r#"
                    INSERT INTO
                        uploads (client_id, metadata_id, reported_size, file_path, state)
                        VALUES ($1, $2, $3, $4, 'started')
                        RETURNING id;
                "#)
                .bind(client_id.to_string())
                .bind(metadata_id.to_string())
                .bind(reported_size as i64)
                .bind(&tmp_upload_path)
                .fetch_one(conn)
                .await
                .map_err(sqlite::map_sqlx_error)
                .map_err(UploadError::Database)?;

            Uuid::parse_str(&bare_upload_id.id).unwrap()
        }
    };

    Ok((upload_id, tmp_upload_path))
}

async fn record_upload_failed(db: &Database, upload_id: Uuid) -> Result<(), UploadError> {
    match db.ex() {
        #[cfg(feature = "postgres")]
        Executor::Postgres(ref mut conn) => {
            use crate::database::postgres;

            let _rows_affected: i32 = sqlx::query_scalar(r#"
                    UPDATE uploads SET state = 'failed' WHERE id = $1;
                "#)
                .bind(upload_id.to_string())
                .fetch_one(conn)
                .await
                .map_err(postgres::map_sqlx_error)?;
        }

        #[cfg(feature = "sqlite")]
        Executor::Sqlite(ref mut conn) => {
            use crate::database::sqlite;

            let _rows_affected: i32 = sqlx::query_scalar(r#"
                    UPDATE uploads SET state = 'failed' WHERE id = $1;
                "#)
                .bind(upload_id.to_string())
                .fetch_one(conn)
                .await
                .map_err(sqlite::map_sqlx_error)?;
        }
    }

    Ok(())
}

struct StoreResult {
    integrity_hash: String,
    size: usize,
}

async fn process_upload_stream<S>(
    mut stream: S,
    writer: &mut Box<dyn AsyncWrite + Unpin + Send>,
) -> Result<StoreResult, UploadStreamError>
where
    S: TryStream<Ok = bytes::Bytes, Error = std::io::Error> + Unpin,
{
    let mut car_buffer = car_buffer::CarBuffer::new();
    let mut hasher = blake3::Hasher::new();
    let mut bytes_written = 0;

    while let Some(chunk) = stream.try_next().await.map_err(UploadStreamError::ReadFailed)? {
        hasher.update(&chunk);
        car_buffer.add_chunk(&chunk);
        bytes_written += chunk.len();

        writer
            .write_all(&chunk)
            .await
            .map_err(UploadStreamError::WriteFailed)?
    }

    let integrity_hash = hasher.finalize().to_string();
    Ok(StoreResult { integrity_hash, size: bytes_written })
}

#[derive(Debug, thiserror::Error)]
pub enum UploadError {
    #[error("a database error occurred during an upload")]
    Database(#[from] DbError),

    #[error("we expected a data field but received nothing")]
    DataFieldMissing,

    #[error("failed to acquire data field from body")]
    DataFieldUnavailable(multer::Error),

    #[error("account is not authorized to store {0} bytes, {1} bytes are still authorized")]
    InsufficientAuthorizedStorage(u64, u64),

    #[error("request's data payload was malformed")]
    InvalidRequestData(multer::Error),

    #[error("failed to acquire request field from body")]
    RequestFieldUnavailable(multer::Error),

    #[error("we expected a request field but received nothing")]
    RequestFieldMissing,

    #[error("streaming car upload failed")]
    StreamingFailed(UploadStreamError),

    #[error("unable to open store for properly authorized data upload: {0}")]
    StoreUnavailable(object_store::Error),
}

impl IntoResponse for UploadError {
    fn into_response(self) -> Response {
        use UploadError::*;

        match self {
            Database(_) | StoreUnavailable(_) => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            DataFieldUnavailable(_) | DataFieldMissing | InvalidRequestData(_) | RequestFieldUnavailable(_) | RequestFieldMissing => {
                let err_msg = serde_json::json!({ "msg": format!("{self}") });
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            InsufficientAuthorizedStorage(_, _) => {
                let err_msg = serde_json::json!({ "msg": format!("{self}") });
                (StatusCode::PAYLOAD_TOO_LARGE, Json(err_msg)).into_response()
            }
            StreamingFailed(stream_err) => stream_err.into_response(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UploadStreamError {
    #[error("failed to read from client")]
    ReadFailed(std::io::Error),

    #[error("failed to write to storage backend")]
    WriteFailed(std::io::Error),
}

impl IntoResponse for UploadStreamError {
    fn into_response(self) -> Response {
        use UploadStreamError::*;

        match self {
            ReadFailed(_) => {
                let err_msg = serde_json::json!({ "msg": format!("client stream went away before file upload was complete") });
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

mod car_buffer {
    use bytes::Bytes;
    use futures::TryStreamExt;

    // We only look through the first MiB for the keys section. It's considered badly formed if not
    // found in that window.
    const SCANNED_CAR_CAPACITY: usize = 1_048_576;

    enum BufferState {
        Buffering(Vec<u8>),
        Done,
        Empty,
    }

    pub struct CarBuffer {
        state: BufferState,
    }

    impl CarBuffer {
        pub fn add_chunk(&mut self, bytes: &Bytes) {
            self.state = match std::mem::replace(&mut self.state, BufferState::Empty) {
                // We don't need any extra data, just return
                BufferState::Done => return,

                // This is the first chunk
                BufferState::Empty => {
                    let data = bytes
                        .clone()
                        .into_iter()
                        .take(SCANNED_CAR_CAPACITY)
                        .collect();
                    BufferState::Buffering(data)
                }
                // We've already some data, we just need to extend it to cover as much as possible
                BufferState::Buffering(mut vec) => {
                    let remaining_space = SCANNED_CAR_CAPACITY - vec.len();
                    let consumable_bytes = remaining_space.min(bytes.len());

                    // Add as many bytes as we can to the buffer
                    vec.extend(bytes.slice(0..consumable_bytes));

                    BufferState::Buffering(vec)
                }
            };
        }

        fn as_slice(&self) -> &[u8] {
            match &self.state {
                BufferState::Buffering(vec) => vec.as_slice(),
                _ => &[],
            }
        }

        fn is_done(&self) -> bool {
            matches!(&self.state, BufferState::Done)
        }

        pub fn is_ready(&self) -> bool {
            matches!(&self.state, BufferState::Buffering(_))
        }

        pub fn new() -> Self {
            Self {
                state: BufferState::Empty,
            }
        }

        pub fn parse(&mut self) -> Result<Option<()>, &str> {
            if self.is_done() {
                return Err("already finished");
            }

            if !self.is_ready() {
                return Ok(None);
            }

            tracing::info!("parsing things out of the car file woo");

            // TODO: process data buffer
            let _ = self.as_slice();
            self.state = BufferState::Done;

            // TODO: return something useful
            Ok(Some(()))
        }
    }
}
