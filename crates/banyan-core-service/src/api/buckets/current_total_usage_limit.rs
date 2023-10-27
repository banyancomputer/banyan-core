use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::extractors::ApiIdentity;

pub async fn handler(_api_id: ApiIdentity) -> Response {
    let size: u64 = 50 * 1024 * 1024 * 1024;
    let resp = serde_json::json!({"size": size });
    (StatusCode::OK, Json(resp)).into_response()
}
