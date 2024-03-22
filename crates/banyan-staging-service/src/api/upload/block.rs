use axum::extract::{BodyStream, State};
use axum::headers::{ContentLength, ContentType};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::TypedHeader;
use banyan_object_store::{ObjectStore, ObjectStorePath};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use super::db::{
    complete_upload, report_upload, upload_size, write_block_to_tables, UPLOAD_SESSION_DURATION,
};
use super::error::UploadError;
use crate::app::AppState;
use crate::database::models::Uploads;
use crate::extractors::AuthenticatedClient;

#[derive(Deserialize, Serialize)]
pub struct BlockUploadRequest {
    cid: String,

    #[serde(flatten, default)]
    details: Option<UploadDetails>,
}

#[derive(Serialize, Deserialize)]
pub struct UploadDetails {
    completed: bool,
    upload_id: String,
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
    let constraints = multer::Constraints::new().allowed_fields(vec!["request-data", "block"]);

    let mut multipart = multer::Multipart::with_constraints(body, boundary, constraints);

    let client_id_str = client.id().to_string();
    let request_field = multipart
        .next_field()
        .await
        .map_err(UploadError::RequestFieldUnavailable)?
        .ok_or(UploadError::RequestFieldMissing)?;

    let request = match request_field.json::<BlockUploadRequest>().await {
        Ok(r) => r,
        Err(err) => {
            tracing::error!("failed to parse upload request: {err}");
            return Err(UploadError::InvalidRequestData(err));
        }
    };

    // todo: the CID crate fails to read non 512 bytes hash CIDs which we don't want to use, we
    // should get a better parser and validate this...
    let normalized_cid = request.cid;

    let mut conn = db.acquire().await?;

    let details = match request.details {
        Some(details) => details,
        // The one off case isn't supported yet
        None => return Err(UploadError::NotSupported),
    };

    let (req_upload_id, completed) = (details.upload_id, details.completed);
    let upload = Uploads::by_id_and_client(&mut *conn, &req_upload_id, &client_id_str).await?;

    let created_at = upload.created_at.ok_or(UploadError::UploadLookupFailure)?;
    if created_at < (OffsetDateTime::now_utc() - UPLOAD_SESSION_DURATION) {
        return Err(UploadError::UploadLookupFailure);
    }

    if upload.id != req_upload_id {
        return Err(UploadError::IdMismatch);
    }

    // Don't allow additional data on a completed upload
    if upload.state == "complete" {
        return Err(UploadError::UploadIsComplete);
    }

    let block_field = multipart
        .next_field()
        .await
        .map_err(UploadError::DataFieldUnavailable)?
        .ok_or(UploadError::DataFieldMissing)?;

    let block: Bytes = block_field
        .bytes()
        .await
        .map_err(UploadError::DataFieldUnavailable)?;

    write_block_to_tables(&mut conn, &upload.id, &normalized_cid, block.len() as i64).await?;

    let location =
        ObjectStorePath::from(format!("{}/{}.bin", upload.base_path, normalized_cid).as_str());

    store
        .put(&location, block)
        .await
        .map_err(UploadError::ObjectStore)?;

    // If we've just finished off the upload, complete and report it
    if completed {
        let total_size = upload_size(&mut conn, &upload.id).await?;
        complete_upload(&mut conn, total_size, "", &upload.id).await?;
        report_upload(
            &mut conn,
            client.storage_grant_id(),
            &upload.metadata_id,
            &upload.id,
            total_size,
        )
        .await?;
    }

    Ok((StatusCode::NO_CONTENT, ()).into_response())
}
