use axum::extract::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::extractors::ApiIdentity;

/// Return the account id of the currently authenticated user
pub async fn handler(api_id: ApiIdentity) -> Response {
    let resp_msg = serde_json::json!({"user_id": api_id.user_id});
    (StatusCode::OK, Json(resp_msg)).into_response()
}
