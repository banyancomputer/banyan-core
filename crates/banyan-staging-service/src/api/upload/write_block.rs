use std::str::FromStr;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use banyan_task::TaskLikeExt;
use bytes::Bytes;
use cid::multibase::Base;
use cid::Cid;
use object_store::path::Path;
use object_store::ObjectStore;
use serde::Deserialize;
use tracing::warn;
use uuid::Uuid;

use super::db::write_block_to_tables;
use crate::api::upload::{complete_upload, get_upload, start_upload};
use crate::app::AppState;
use crate::extractors::AuthenticatedClient;
use crate::tasks::ReportUploadTask;
use crate::upload_store::UploadStore;

#[axum::debug_handler]
pub async fn handler(
    State(state): State<AppState>,
    client: AuthenticatedClient,
    store: UploadStore,
    Json(request): Json<BlockWriteRequest>,
) -> Result<Response, BlockWriteError> {
    let mut db = state.database();
    // let cid = Cid::read_bytes(&request.data[..]).map_err(BlockWriteError::ComputeCid)?;
    // tracing::info!("yippeeeee: {cid}");
    // if cid != request.cid {
    //     return Err(BlockWriteError::MismatchedCid((request.cid, cid)));
    // }

    let normalized_cid = request
        .cid
        .to_string_of_base(Base::Base64Url)
        .map_err(BlockWriteError::ComputeCid)?;

    // Get or create the Upload object associated with this write request
    let maybe_upload = get_upload(&db, client.id(), request.metadata_id)
        .await
        .map_err(BlockWriteError::DbFailure)?;

    let upload = match maybe_upload {
        Some(upload) => upload,
        None => start_upload(&db, &client.id(), &request.metadata_id, 0)
            .await
            .map_err(BlockWriteError::DbFailure)?,
    };

    let blocks_path: String = upload.blocks_path;
    if blocks_path.to_lowercase().ends_with(".car") {
        return Err(BlockWriteError::CarFile);
    }

    write_block_to_tables(
        &db,
        &upload.id,
        &normalized_cid,
        request.data.len() as i64,
        1,
    )
    .await
    .map_err(BlockWriteError::DbFailure)?;

    // Actually write the bytes to the expected location
    let location = Path::from(format!("{blocks_path}/{normalized_cid}.block").as_str());
    store
        .put(&location, Bytes::copy_from_slice(request.data.as_slice()))
        .await
        .map_err(BlockWriteError::WriteFailed)?;

    tracing::error!("rqcpmlted: {:?}", request.completed);

    // If the client marked this request as being the final one in the upload
    if request.completed.is_some() {
        complete_upload(&db, 0, "", &upload.id)
            .await
            .map_err(BlockWriteError::DbFailure)?;

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
        .await
        .map_err(BlockWriteError::DbFailure)?;

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
        .await
        .map_err(BlockWriteError::DbFailure)?;

        ReportUploadTask::new(
            client.storage_grant_id(),
            request.metadata_id,
            &all_cids,
            total_size as u64,
        )
        .enqueue::<banyan_task::SqliteTaskStore>(&mut db)
        .await
        .map_err(|_| BlockWriteError::CarFile)?;
    }

    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum BlockWriteError {
    #[error("internal database error occurred")]
    DbFailure(sqlx::Error),

    #[error("Data in request mismatched attached CID")]
    MismatchedCid((Cid, Cid)),

    #[error("Failed to compute CID")]
    ComputeCid(cid::Error),

    #[error("failed to write to storage backend")]
    WriteFailed(object_store::Error),

    #[error("cannot write Blocks to CAR files")]
    CarFile,
}

impl IntoResponse for BlockWriteError {
    fn into_response(self) -> Response {
        match self {
            BlockWriteError::DbFailure(err) => {
                warn!("db failure writing block: {err}");
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            BlockWriteError::MismatchedCid((expected, actual)) => {
                warn!("block write data didn't match expected cid.\nexpected:\t{expected}\nactual:\t{actual}");
                let err_msg = serde_json::json!({ "msg": format!("block / data mismatch") });
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            BlockWriteError::ComputeCid(err) => {
                warn!("failed to compute CID for some data: {err}");
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            BlockWriteError::WriteFailed(err) => {
                warn!("failed to write individual Block to backend store: {err}");
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            BlockWriteError::CarFile => {
                warn!(
                    "unable to write new blocks to CAR files. create a new upload on the new API."
                );
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}

#[derive(Deserialize)]
pub struct BlockWriteRequest {
    pub cid: Cid,
    pub data: Vec<u8>,
    pub metadata_id: Uuid,
    pub completed: Option<bool>,
}
