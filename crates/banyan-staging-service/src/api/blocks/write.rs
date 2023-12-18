use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use cid::Cid;
use serde::Deserialize;

use crate::app::AppState;
// use crate::database::models::BlockDetails;
// use crate::database::Database;
use crate::extractors::BlockReader;
use crate::upload_store::UploadStore;

#[axum::debug_handler]
pub async fn handler(
    State(state): State<AppState>,
    _client: BlockReader,
    _store: UploadStore,
    Json(request): Json<BlockWriteRequest>,
) -> Result<Response, BlockWriteError> {
    let mut _db = state.database();
    let cid = Cid::read_bytes(&request.data[..]).map_err(BlockWriteError::ComputeCid)?;
    if cid != request.cid {
        return Err(BlockWriteError::CidMismatch((request.cid, cid)));
    }

    // Perform some steps to ensure the user is authenticated

    // Go fetch the CID from the file location

    Ok((StatusCode::OK, ()).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum BlockWriteError {
    #[error("internal database error occurred")]
    DbFailure(sqlx::Error),

    #[error("Data in request mismatched attached CID")]
    CidMismatch((Cid, Cid)),

    #[error("Failed to compute CID")]
    ComputeCid(cid::Error),
}

impl IntoResponse for BlockWriteError {
    fn into_response(self) -> Response {
        match self {
            BlockWriteError::DbFailure(err) => {
                tracing::warn!("db failure writing block: {err}");
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            BlockWriteError::CidMismatch((expected, actual)) => {
                tracing::warn!("block write data didn't match expected cid.\nexpected:\t{expected}\nactual:\t{actual}");
                let err_msg = serde_json::json!({ "msg": format!("block / data mismatch") });
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            BlockWriteError::ComputeCid(err) => {
                tracing::warn!("failed to compute CID for some data: {err}");
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
    pub completed: Option<()>,
}
