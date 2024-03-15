use axum::extract::{BodyStream, State};
use axum::headers::{ContentLength, ContentType};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::TypedHeader;
use banyan_car_analyzer::{CarReport, StreamingCarAnalyzer, StreamingCarAnalyzerError};
use banyan_object_store::{ObjectStore, ObjectStorePath};
use banyan_task::TaskLikeExt;
use futures::{TryStream, TryStreamExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use crate::database::Database;
use crate::extractors::AuthenticatedClient;
use crate::tasks::ReportUploadTask;
pub(crate) mod block;
mod db;
mod error;
pub(crate) mod new;

pub use db::{
    complete_upload, fail_upload, start_upload, upload_size, write_block_to_tables, Upload,
};
pub use error::UploadError;

/// Limit on the size of the JSON request that accompanies an upload.
const UPLOAD_REQUEST_SIZE_LIMIT: u64 = 100 * 1_024;

#[derive(Deserialize, Serialize)]
pub struct CarUploadRequest {
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
    let db = state.database();

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

    let request: CarUploadRequest = request_field
        .json()
        .await
        .map_err(UploadError::InvalidRequestData)?;
    let content_hash = request.content_hash;

    let upload = start_upload(
        &db,
        &client.id().to_string(),
        &request.metadata_id.to_string(),
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

    match process_upload_stream(
        &db,
        &upload,
        store,
        reported_body_length as usize,
        content_hash,
        car_field,
    )
    .await
    {
        Ok(cr) => {
            complete_upload(&db, 0, cr.integrity_hash(), &upload.id).await?;

            let mut conn = db.acquire().await?;

            ReportUploadTask::new(
                client.storage_grant_id(),
                &request.metadata_id.to_string(),
                cr.cids(),
                cr.total_size(),
            )
            .enqueue::<banyan_task::SqliteTaskStore>(&mut conn)
            .await
            .map_err(UploadError::FailedToEnqueueTask)?;

            Ok((StatusCode::NO_CONTENT, ()).into_response())
        }
        Err(err) => {
            // todo: we don't care in the response if this fails, but if it does we will want to
            // clean it up in the future which should be handled by a background task
            let _ = fail_upload(&db, &upload.id).await;
            Err(err)
        }
    }
}

async fn process_upload_stream<S>(
    db: &Database,
    upload: &Upload,
    store: ObjectStore,
    expected_size: usize,
    content_hash: String,
    mut stream: S,
) -> Result<CarReport, UploadError>
where
    S: TryStream<Ok = bytes::Bytes, Error = multer::Error> + Unpin,
{
    let mut car_analyzer = StreamingCarAnalyzer::new();
    let mut warning_issued = false;
    let mut hasher = blake3::Hasher::new();
    while let Some(chunk) = stream.try_next().await.map_err(UploadError::ReadFailed)? {
        hasher.update(&chunk);
        car_analyzer.add_chunk(&chunk)?;
        while let Some(block) = car_analyzer.next().await? {
            let cid_string = block
                .cid()
                .to_string_of_base(cid::multibase::Base::Base64Url)
                .expect("parsed cid to unparse");

            let file_path = format!("{}/{}.bin", upload.base_path, cid_string);
            let obj_path = ObjectStorePath::from(file_path);
            let length = block.length() as i64;

            store
                .put(&obj_path, bytes::Bytes::from(block.into_data()))
                .await
                .map_err(UploadError::ObjectStore)?;

            write_block_to_tables(db, &upload.id, &cid_string, length).await?;
        }

        if car_analyzer.seen_bytes() as usize > expected_size && !warning_issued {
            warning_issued = true;
            tracing::warn!(
                "client is streaming more data than was claimed to be present in the upload"
            );
        }
    }

    if hasher.finalize().to_string() != content_hash {
        return Err(UploadError::ParseError(
            StreamingCarAnalyzerError::MismatchedHash,
        ));
    }

    Ok(car_analyzer.report()?)
}
