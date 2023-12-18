use axum::extract::{BodyStream, State};
use axum::headers::{ContentLength, ContentType};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::TypedHeader;
use banyan_car_analyzer::{CarReport, StreamingCarAnalyzer, StreamingCarAnalyzerError};
use banyan_task::TaskLikeExt;
use bytes::Bytes;
use futures::{TryStream, TryStreamExt};
use object_store::path::Path;
use serde::{Deserialize, Serialize};
use tracing::warn;
use uuid::Uuid;

use crate::app::AppState;
use crate::database::{map_sqlx_error, Database};
use crate::extractors::AuthenticatedClient;
use crate::tasks::ReportUploadTask;
use crate::upload_store::{ObjectStore, UploadStore};

pub(crate) mod db_helpers;
mod error;
pub(crate) mod write_block;
use db_helpers::*;
use error::{UploadError, UploadStreamError};

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
    store: UploadStore,
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

    let upload = start_upload(
        &db,
        &client.id(),
        &request.metadata_id,
        reported_body_length,
    )
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

    // let store_path = Path::from(tmp_upload_dir.as_str());

    warn!("about to process the stream");

    match process_upload_stream(
        &db,
        &upload,
        &store,
        reported_body_length as usize,
        car_field,
        content_hash,
    )
    .await
    {
        Ok(cr) => {
            complete_upload(&db, cr.total_size() as i64, cr.integrity_hash(), &upload.id).await?;
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
            handle_failed_upload(&db, &upload.id).await;
            Err(err.into())
        }
    }
}

async fn handle_failed_upload(db: &Database, upload_id: &str) {
    // attempt to report the upload as failed, but that fails we'll need to handle it in a
    // future clean-up task. todo: should actually just enqueue and entire clean up process
    // and report this as failed there...
    let _ = fail_upload(db, upload_id).await;
}

async fn process_upload_stream<S>(
    db: &Database,
    upload: &Upload,
    store: &UploadStore,
    expected_size: usize,
    mut stream: S,
    content_hash: String,
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
        while let Some(block) = car_analyzer.next().await? {
            let cid_string = block
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
            .bind(block.length() as i64)
            .execute(db)
            .await
            .map_err(map_sqlx_error)?;

            let block_id: Uuid = {
                let cid_id: String =
                    sqlx::query_scalar("SELECT id FROM blocks WHERE cid = $1 LIMIT 1;")
                        .bind(cid_string.clone())
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
            .bind(upload.id.clone())
            .bind(block_id.to_string())
            .bind(block.offset() as i64)
            .execute(db)
            .await
            .map_err(map_sqlx_error)?;

            let location =
                Path::from(format!("{}/{}.block", upload.blocks_path, cid_string).as_str());
            store
                .put(&location, Bytes::copy_from_slice(&block.data()[..]))
                .await
                .map_err(UploadStreamError::WriteFailed)?;
        }

        if car_analyzer.seen_bytes() as usize > expected_size && !warning_issued {
            warning_issued = true;
            tracing::warn!(
                "client is streaming more data than was claimed to be present in the upload"
            );
        }
    }

    if hasher.finalize().to_string() != content_hash {
        return Err(UploadStreamError::ParseError(
            StreamingCarAnalyzerError::MismatchedHash,
        ));
    }

    Ok(car_analyzer.report()?)
}
