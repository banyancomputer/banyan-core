use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::extractors::UserIdentity;

pub async fn handler(_user_id: UserIdentity) -> Response {
    let size: u64 = 50 * 1024 * 1024 * 1024;
    let resp = serde_json::json!({"size": size });
    (StatusCode::OK, Json(resp)).into_response()
}
