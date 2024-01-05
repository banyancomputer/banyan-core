use std::str::FromStr;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use banyan_task::TaskLikeExt;
use bytes::Bytes;
use cid::{
    multibase::Base,
    multihash::{Code, MultihashDigest},
    Cid,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::api::upload::{
    complete_upload, get_upload, start_upload, write_block_to_tables, UploadError,
};
use crate::app::AppState;
use crate::extractors::AuthenticatedClient;
use crate::tasks::ReportUploadTask;
use banyan_object_store::{ObjectStore, ObjectStorePath};

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

    /*
    let blocks_path: String = upload.blocks_path;
    if blocks_path.to_lowercase().ends_with(".car") {
        return Err(UploadError::CarFile);
    }
    */

    write_block_to_tables(
        &db,
        &upload.id,
        &normalized_cid,
        request.data.len() as i64,
        1,
    )
    .await?;

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

        let all_cids: Vec<String> = sqlx::query_scalar(
            r#"
            SELECT blocks.cid 
            FROM blocks
            JOIN uploads_blocks ON blocks.id = uploads_blocks.block_id
            JOIN uploads ON uploads_blocks.upload_id = uploads.id
            WHERE uploads.id = $1;
            "#,
        )
        .bind(&upload.id)
        .fetch_all(&db)
        .await?;

        let all_cids = all_cids
            .into_iter()
            .map(|cid_string| Cid::from_str(&cid_string).unwrap())
            .collect::<Vec<Cid>>();

        let total_size: i64 = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(blocks.data_length), 0)
            FROM blocks
            JOIN uploads_blocks ON blocks.id = uploads_blocks.block_id
            JOIN uploads ON uploads_blocks.upload_id = uploads.id
            WHERE uploads.id = $1;
            "#,
        )
        .bind(&upload.id)
        .fetch_one(&db)
        .await?;

        ReportUploadTask::new(
            client.storage_grant_id(),
            request.metadata_id,
            &all_cids,
            total_size as u64,
        )
        .enqueue::<banyan_task::SqliteTaskStore>(&mut db)
        .await
        .map_err(|_| UploadError::CarFile)?;
    }

    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

#[derive(Deserialize)]
pub struct BlockWriteRequest {
    pub cid: Cid,
    pub data: Vec<u8>,
    pub metadata_id: Uuid,
    pub completed: Option<bool>,
}
