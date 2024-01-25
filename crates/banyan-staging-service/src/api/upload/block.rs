use axum::extract::{BodyStream, State};
use axum::headers::{ContentLength, ContentType};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Json, TypedHeader};
use banyan_car_analyzer::{CarReport, StreamingCarAnalyzer, StreamingCarAnalyzerError};
use banyan_object_store::{ObjectStore, ObjectStorePath};
use banyan_task::TaskLikeExt;
use bytes::Bytes;
use cid::multibase::Base;
use cid::multihash::{Code, MultihashDigest};
use cid::Cid;
use futures::{TryStream, TryStreamExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::db::{
    complete_upload, fail_upload, get_upload, report_upload, start_upload, write_block_to_tables,
    Upload,
};
use super::error::UploadError;
use crate::app::AppState;
use crate::database::Database;
use crate::extractors::AuthenticatedClient;
use crate::tasks::ReportUploadTask;

#[derive(Deserialize, Serialize)]
pub struct BlockUploadRequest {
    metadata_id: Uuid,
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
    let constraints = multer::Constraints::new();
    //.allowed_fields(vec!["request-data", "car-upload"])

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

    // Get the upload either new or ongoing
    let (upload, completed) = match request.details {
        // This request is the start and end of this block upload
        BlockUploadDetails::OneOff => {
            let upload = start_upload(
                &db,
                &client.id(),
                &request.metadata_id,
                reported_body_length,
            )
            .await?;
            (upload, true)
        }
        // We're in the middle of a multi-request block writing sequence
        BlockUploadDetails::Ongoing {
            completed,
            upload_id,
        } => {
            // Assume that the upload has already been created via the `new` endpoint
            let upload = get_upload(&db, client.id(), request.metadata_id)
                .await?
                .unwrap();
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

        // Compute the cid associated with that block to verify data integrity
        let codec = request.cid.codec();
        let hash = Code::Sha2_256.digest(&block);
        let computed_cid = Cid::new(cid::Version::V1, codec, hash)
            .map_err(UploadError::Cid)?
            .to_string_of_base(Base::Base64Url)
            .map_err(UploadError::Cid)?;
        let normalized_cid = request
            .cid
            .to_string_of_base(Base::Base64Url)
            .map_err(UploadError::Cid)?;
        if computed_cid != normalized_cid {
            return Err(UploadError::MismatchedCid((normalized_cid, computed_cid)));
        }
        // Write this block to the tables
        write_block_to_tables(&db, &upload.id, &normalized_cid, block.len() as i64).await?;

        // Write the bytes to the expected location
        let location = ObjectStorePath::from(
            format!("{}/{}.block", request.metadata_id, normalized_cid).as_str(),
        );
        store
            .put(&location, block)
            .await
            .map_err(UploadError::ObjectStore)?;
    }

    // If we've just finished off the upload, complete and report it
    if completed {
        complete_upload(&db, 0, "", &upload.id).await?;
        report_upload(
            &db,
            client.storage_grant_id(),
            request.metadata_id,
            &upload.id,
        )
        .await?;
    }

    Ok((StatusCode::NO_CONTENT, ()).into_response())
}
