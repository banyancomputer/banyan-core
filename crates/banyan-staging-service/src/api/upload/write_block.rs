use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use bytes::Bytes;
use cid::multibase::Base;
use cid::Cid;
use object_store::path::Path;
use object_store::ObjectStore;
use serde::Deserialize;
use uuid::Uuid;

use super::super::upload::db_helpers::*;
use crate::app::AppState;
// use crate::database::models::BlockDetails;
// use crate::database::Database;
use crate::extractors::AuthenticatedClient;
use crate::upload_store::UploadStore;

#[axum::debug_handler]
pub async fn handler(
    State(state): State<AppState>,
    client: AuthenticatedClient,
    store: UploadStore,
    Json(request): Json<BlockWriteRequest>,
) -> Result<Response, BlockWriteError> {
    let db = state.database();
    let cid = Cid::read_bytes(&request.data[..]).map_err(BlockWriteError::ComputeCid)?;
    if cid != request.cid {
        return Err(BlockWriteError::MismatchedCid((request.cid, cid)));
    }
    let normalized_cid = cid
        .to_string_of_base(Base::Base64Url)
        .map_err(BlockWriteError::ComputeCid)?;

    // Get or create the Upload object associated with this write request
    let upload = get_upload(&db, client.id(), request.metadata_id)
        .await
        .unwrap()
        .unwrap_or(
            start_upload(&db, &client.id(), &request.metadata_id, 0)
                .await
                .unwrap(),
        );

    let blocks_path: String = upload.blocks_path;
    if blocks_path.to_lowercase().ends_with(".car") {
        return Err(BlockWriteError::CarFile);
    }

    // Actually write the bytes to the expected location
    let location = Path::from(format!("{blocks_path}/{normalized_cid}.block").as_str());
    store
        .put(&location, Bytes::copy_from_slice(request.data.as_slice()))
        .await
        .map_err(BlockWriteError::WriteFailed)?;

    // If the client marked this request as being the final one in the upload
    if request.completed.is_some() {
        complete_upload(&db, 0, "", &upload.id).await.unwrap();
    }

    Ok((StatusCode::OK, ()).into_response())
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
                tracing::warn!("db failure writing block: {err}");
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            BlockWriteError::MismatchedCid((expected, actual)) => {
                tracing::warn!("block write data didn't match expected cid.\nexpected:\t{expected}\nactual:\t{actual}");
                let err_msg = serde_json::json!({ "msg": format!("block / data mismatch") });
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            BlockWriteError::ComputeCid(err) => {
                tracing::warn!("failed to compute CID for some data: {err}");
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            BlockWriteError::WriteFailed(err) => {
                tracing::warn!("failed to write individual Block to backend store: {err}");
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            BlockWriteError::CarFile => {
                tracing::warn!(
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
    pub completed: Option<()>,
}
