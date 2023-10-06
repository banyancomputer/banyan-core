use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::extractors::ApiToken;

pub async fn handler(_api_token: ApiToken) -> Response {
    let resp = serde_json::json!({"size":  50 * 1024 * 1024 * 1024 });
    (StatusCode::OK, Json(resp)).into_response()
}
