use axum::extract::{BodyStream, State};
use axum::headers::{ContentLength, ContentType};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::TypedHeader;
use banyan_object_store::{ObjectStore, ObjectStorePath};
use bytes::Bytes;
use cid::multibase::Base;
use cid::Cid;
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
    cid: Cid,

    // Optional additional details about the nature of the upload
    #[serde(flatten)]
    details: BlockUploadDetails,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum BlockUploadDetails {
    Ongoing { completed: bool, upload_id: String },
    OneOff,
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

    // Grab the request object
    let request: BlockUploadRequest = multipart
        .next_field()
        .await
        .map_err(UploadError::RequestFieldUnavailable)?
        .ok_or(UploadError::RequestFieldMissing)?
        .json()
        .await
        .map_err(UploadError::InvalidRequestData)?;

    let mut conn = db.acquire().await?;

    // Get the upload either new or ongoing
    let (upload, completed) = match request.details {
        // This request is the start and end of this block upload
        BlockUploadDetails::OneOff => {
            // TODO there isn't currently a way to start uploads without having an
            // associated metadata_id. If future OneOff requests are to exist outside
            // of the context of our pipelines, this needs to change.
            return Err(UploadError::NotSupported);
        }
        // We're in the middle of a multi-request block writing sequence
        BlockUploadDetails::Ongoing {
            completed,
            upload_id,
        } => {
            let client_id_str = client.id().to_string();
            tracing::warn!(client_id_str, upload_id, completed, "looking for upload...");
            let upload = Uploads::by_id_and_client(&mut *conn, &upload_id, &client_id_str).await?;

            let created_at = upload.created_at.ok_or(UploadError::UploadLookupFailure)?;
            if created_at < (OffsetDateTime::now_utc() - UPLOAD_SESSION_DURATION) {
                return Err(UploadError::UploadLookupFailure);
            }

            if upload.id != upload_id {
                return Err(UploadError::IdMismatch);
            }

            (upload, completed)
        }
    };
    // If the upload had already been marked as complete
    if upload.state == "complete" {
        return Err(UploadError::UploadIsComplete);
    }

    // While there are still block fields encoded
    while let Some(block_field) = multipart
        .next_field()
        .await
        .map_err(UploadError::DataFieldUnavailable)?
    {
        // Grab all of the block data from this request part
        let block: Bytes = block_field
            .bytes()
            .await
            .map_err(UploadError::DataFieldUnavailable)?;

        let normalized_cid = request
            .cid
            .to_string_of_base(Base::Base64Url)
            .map_err(UploadError::Cid)?;

        // Write this block to the tables
        write_block_to_tables(&mut conn, &upload.id, &normalized_cid, block.len() as i64).await?;

        // Write the bytes to the expected location
        let location =
            ObjectStorePath::from(format!("{}/{}.bin", upload.base_path, normalized_cid).as_str());
        store
            .put(&location, block)
            .await
            .map_err(UploadError::ObjectStore)?;
    }

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
