use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

use crate::app::State as AppState;
use crate::extractors::{Database, StorageGrant};

#[derive(Deserialize, Serialize)]
pub struct GrantRequest {
    public_key: String,
}

#[axum::debug_handler]
pub async fn handler(
    // this weirdly needs to be present even though we don't use it
    State(_state): State<AppState>,
    database: Database,
    grant: StorageGrant,
    Json(_request): Json<GrantRequest>,
) -> Response {
    let msg = serde_json::json!({"msg": "success"});
    (StatusCode::NO_CONTENT, axum::Json(msg)).into_response()
}
