use std::str::FromStr;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use banyan_object_store::{ObjectStore, ObjectStorePath};
use banyan_task::TaskLikeExt;
use bytes::Bytes;
use cid::multibase::Base;
use cid::multihash::{Code, MultihashDigest};
use cid::Cid;
use serde::Deserialize;
use uuid::Uuid;

use crate::api::upload::{
    complete_upload, get_upload, report_upload, start_upload, write_block_to_tables, UploadError,
};
use crate::app::AppState;
use crate::extractors::AuthenticatedClient;
use crate::tasks::ReportUploadTask;

#[axum::debug_handler]
pub async fn handler(
    State(state): State<AppState>,
    client: AuthenticatedClient,
    store: ObjectStore,
    Json(request): Json<BlockWriteRequest>,
) -> Result<Response, UploadError> {
    let mut db = state.database();
    let codec = request.cid.codec();
    let hash = Code::Sha2_256.digest(&request.data);
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

    // Get or create the Upload object associated with this write request
    let maybe_upload = get_upload(&db, client.id(), request.metadata_id).await?;

    let upload = match maybe_upload {
        Some(upload) => upload,
        None => start_upload(&db, &client.id(), &request.metadata_id, 0).await?,
    };

    if upload.state == "complete" {
        return Err(UploadError::UploadIsComplete);
    }

    write_block_to_tables(&db, &upload.id, &normalized_cid, request.data.len() as i64).await?;

    // Actually write the bytes to the expected location
    let location =
        ObjectStorePath::from(format!("{}/{}.block", request.metadata_id, normalized_cid).as_str());
    store
        .put(&location, Bytes::copy_from_slice(request.data.as_slice()))
        .await
        .map_err(UploadError::ObjectStore)?;

    // If the client marked this request as being the final one in the upload
    if request.completed.is_some() {
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

#[derive(Deserialize)]
pub struct BlockWriteRequest {
    pub cid: Cid,
    pub data: Vec<u8>,
    // upload_id which is fed to the client after a preliminary request?
    pub metadata_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<bool>,
}
