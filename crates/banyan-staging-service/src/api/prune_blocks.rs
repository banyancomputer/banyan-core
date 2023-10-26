use axum::extract::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use cid::multibase::Base;
use cid::Cid;
use jwt_simple::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::PlatformAuthKey;
use crate::car_analyzer::{CarReport, StreamingCarAnalyzer, StreamingCarAnalyzerError};
use crate::database::{BareId, SqlxError};
use crate::extractors::CoreIdentity;

/// Limit on the size of the JSON request that accompanies an upload.
const UPLOAD_REQUEST_SIZE_LIMIT: u64 = 100 * 1_024;

#[derive(Deserialize)]
pub struct PruneBlock {
    pub normalized_cid: String,
    pub metadata_id: Uuid,
}

pub async fn handler(
    _ci: CoreIdentity,
    Json(_prune_blocks): Json<Vec<PruneBlock>>,
) -> Result<Response, PruneBlocksError> {
    Ok((StatusCode::OK, Json(serde_json::json!({}))).into_response())
}

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum PruneBlocksError {}

impl IntoResponse for PruneBlocksError {
    fn into_response(self) -> Response {
        use PruneBlocksError::*;
        match self {
            _ => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({ "msg": "a backend service issue occurred" });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
