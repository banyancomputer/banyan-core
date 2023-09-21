use std::path::PathBuf;

use axum::extract::{BodyStream, Json};
use axum::headers::{ContentLength, ContentType};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::TypedHeader;
use futures::{TryFutureExt, TryStream, TryStreamExt};
use jwt_simple::prelude::*;
use object_store::ObjectStore;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncWrite, AsyncWriteExt};
use uuid::Uuid;

use crate::app::PlatformAuthKey;
use crate::car_analyzer::{CarReport, StreamingCarAnalyzer, StreamingCarAnalyzerError};
use crate::database::{BareId, DbError, Executor};
use crate::extractors::{AuthenticatedClient, Database, UploadStore};

/// Limit on the size of the JSON request that accompanies an upload.
const UPLOAD_REQUEST_SIZE_LIMIT: u64 = 100 * 1_024;

#[derive(Deserialize, Serialize)]
pub struct UploadRequest {
    metadata_id: Uuid,
    content_hash: String,
}

pub async fn handler(
    db: Database,
    client: AuthenticatedClient,
    store: UploadStore,
    auth_key: PlatformAuthKey,
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

    let store_path = object_store::path::Path::from(tmp_file_path.as_str());
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
            // todo: should be a background task
            report_upload_to_platform(auth_key, request.metadata_id, &cr).await?;

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
    _store: &UploadStore,
    car_report: &CarReport,
    upload_id: &str,
    _file_path: &object_store::path::Path,
) -> Result<(), UploadError> {
    //let mut path_iter = file_path.parts();
    // discard the uploading/ prefix
    //let _ = path_iter.next();
    //let mut final_path: object_store::path::Path = path_iter.collect();

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

    match db.ex() {
        #[cfg(feature = "postgres")]
        Executor::Postgres(ref mut conn) => {
            use crate::database::postgres;

            sqlx::query(
                r#"
                    UPDATE uploads SET
                            state = 'complete',
                            final_size = $1,
                            integrity_hash  = $2
                        WHERE id = $3::uuid;
                "#,
            )
            .bind(car_report.total_size() as i64)
            .bind(car_report.integrity_hash())
            .bind(upload_id)
            .execute(conn)
            .await
            .map_err(postgres::map_sqlx_error)?;
        }

        #[cfg(feature = "sqlite")]
        Executor::Sqlite(ref mut conn) => {
            use crate::database::sqlite;

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
            .execute(conn)
            .await
            .map_err(sqlite::map_sqlx_error)?;
        }
    }

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

    let upload_id = match db.ex() {
        #[cfg(feature = "postgres")]
        Executor::Postgres(ref mut conn) => {
            use crate::database::postgres;

            sqlx::query_scalar(
                r#"
                    INSERT INTO
                        uploads (client_id, metadata_id, reported_size, file_path, state)
                        VALUES ($1::uuid, $2::uuid, $3, $4, 'started')
                        RETURNING CAST(id AS TEXT) as id;
                "#,
            )
            .bind(client_id.to_string())
            .bind(metadata_id.to_string())
            .bind(reported_size as i64)
            .bind(&tmp_upload_path)
            .fetch_one(conn)
            .await
            .map_err(postgres::map_sqlx_error)
            .map_err(UploadError::Database)?
        }

        #[cfg(feature = "sqlite")]
        Executor::Sqlite(ref mut conn) => {
            use crate::database::sqlite;

            sqlx::query_scalar(
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
            .fetch_one(conn)
            .await
            .map_err(sqlite::map_sqlx_error)
            .map_err(UploadError::Database)?
        }
    };

    Ok((upload_id, tmp_upload_path))
}

async fn record_upload_failed(db: &Database, upload_id: &str) -> Result<(), UploadError> {
    match db.ex() {
        #[cfg(feature = "postgres")]
        Executor::Postgres(ref mut conn) => {
            use crate::database::postgres;

            let _rows_affected: i32 = sqlx::query_scalar(
                r#"
                    UPDATE uploads SET state = 'failed' WHERE id = $1::uuid;
                "#,
            )
            .bind(upload_id)
            .fetch_one(conn)
            .await
            .map_err(postgres::map_sqlx_error)?;
        }

        #[cfg(feature = "sqlite")]
        Executor::Sqlite(ref mut conn) => {
            use crate::database::sqlite;

            let _rows_affected: i32 = sqlx::query_scalar(
                r#"
                    UPDATE uploads SET state = 'failed' WHERE id = $1;
                "#,
            )
            .bind(upload_id)
            .fetch_one(conn)
            .await
            .map_err(sqlite::map_sqlx_error)?;
        }
    }

    Ok(())
}

use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, Url};

#[derive(Serialize)]
struct MetadataSizeRequest {
    data_size: u64,
}

async fn report_upload_to_platform(
    auth_key: PlatformAuthKey,
    metadata_id: Uuid,
    report: &CarReport,
) -> Result<(), UploadError> {
    let metadata_size = MetadataSizeRequest {
        data_size: report.total_size(),
    };

    let mut default_headers = HeaderMap::new();
    default_headers.insert("Content-Type", HeaderValue::from_static("application/json"));

    let client = Client::builder()
        .default_headers(default_headers)
        .build()
        .unwrap();

    let report_endpoint = auth_key
        .base_url()
        .join(format!("/api/v1/storage/{}", metadata_id).as_str())
        .unwrap();

    let mut claims = Claims::create(Duration::from_secs(60))
        .with_audiences(HashSet::from_strings(&["banyan-platform"]))
        .with_subject("banyan-staging")
        .invalid_before(Clock::now_since_epoch() - Duration::from_secs(30));

    claims.create_nonce();
    claims.issued_at = Some(Clock::now_since_epoch());

    let bearer_token = auth_key.sign(claims).unwrap();

    let request = client
        .post(report_endpoint)
        .json(&metadata_size)
        .bearer_auth(bearer_token);

    let response = match request.send().await {
        Ok(resp) => resp,
        Err(err) => {
            tracing::error!("failed to send confirmation request to the banyan-platform: {err}");
            return Err(UploadError::FailedReport(bytes::Bytes::from("unable to connect")));
        }
    };

    if response.status().is_success() {
        Ok(())
    } else {
        Err(UploadError::FailedReport(response.bytes().await.unwrap()))
    }
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

            let block_id: Uuid = match db.ex() {
                #[cfg(feature = "postgres")]
                Executor::Postgres(ref mut conn) => {
                    use crate::database::postgres;

                    let cid_id: String = sqlx::query_scalar(
                        r#"
                            INSERT OR IGNORE INTO
                                blocks (cid, data_length)
                                VALUES ($1, $2);
                            SELECT CAST(id AS TEXT) as id FROM blocks WHERE cid = $1 LIMIT 1;
                        "#,
                    )
                    .bind(cid_string)
                    .bind(block_meta.length() as i64)
                    .fetch_one(conn)
                    .await
                    .map_err(postgres::map_sqlx_error)?;

                    // todo: need to support the case where the block already exists...

                    Uuid::parse_str(&cid_id)
                        .map_err(|_| UploadStreamError::DatabaseCorruption("cid uuid parsing"))?
                }

                #[cfg(feature = "sqlite")]
                Executor::Sqlite(ref mut conn) => {
                    use crate::database::sqlite;

                    let cid_id: String = sqlx::query_scalar(
                        r#"
                            INSERT INTO
                                blocks (cid, data_length)
                                VALUES ($1, $2)
                                RETURNING id;
                        "#,
                    )
                    .bind(cid_string)
                    .bind(block_meta.length() as i64)
                    .fetch_one(conn)
                    .await
                    .map_err(sqlite::map_sqlx_error)?;

                    // todo: need to support the case where the block already exists...

                    Uuid::parse_str(&cid_id)
                        .map_err(|_| UploadStreamError::DatabaseCorruption("cid uuid parsing"))?
                }
            };

            // create uploads_blocks row with the block information
            match db.ex() {
                #[cfg(feature = "postgres")]
                Executor::Postgres(ref mut conn) => {
                    use crate::database::postgres;

                    sqlx::query(
                        r#"
                                INSERT INTO
                                    uploads_blocks (upload_id, block_id, byte_offset)
                                    VALUES ($1::uuid, $2::uuid, $3);
                            "#,
                    )
                    .bind(upload_id)
                    .bind(block_id.to_string())
                    .bind(block_meta.offset() as i64)
                    .execute(conn)
                    .await
                    .map_err(postgres::map_sqlx_error)?;
                }

                #[cfg(feature = "sqlite")]
                Executor::Sqlite(ref mut conn) => {
                    use crate::database::sqlite;

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
                    .execute(conn)
                    .await
                    .map_err(sqlite::map_sqlx_error)?;
                }
            };
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
    Database(#[from] DbError),

    #[error("we expected a data field but received nothing")]
    DataFieldMissing,

    #[error("failed to acquire data field from body")]
    DataFieldUnavailable(multer::Error),

    #[error("failed to report upload status to platform")]
    FailedReport(bytes::Bytes),

    #[error("account is not authorized to store {0} bytes, {1} bytes are still authorized")]
    InsufficientAuthorizedStorage(u64, u64),

    #[error("request's data payload was malformed")]
    InvalidRequestData(multer::Error),

    #[error("failed to acquire request field from body")]
    RequestFieldUnavailable(multer::Error),

    #[error("we expected a request field but received nothing")]
    RequestFieldMissing,

    #[error("unable to open store for properly authorized data upload: {0}")]
    StoreUnavailable(object_store::Error),

    #[error("streaming car upload failed")]
    StreamingFailed(#[from] UploadStreamError),
}

impl IntoResponse for UploadError {
    fn into_response(self) -> Response {
        use UploadError::*;

        match self {
            Database(_) | FailedReport(_) | StoreUnavailable(_) => {
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
    #[error("encountered invalid data from the database for {0}")]
    DatabaseCorruption(&'static str),

    #[error("unable to record details about the stream in the database")]
    DatabaseFailure(#[from] DbError),

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
