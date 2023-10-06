use axum::extract::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::extractors::ApiToken;

/// Return the account id of the currently authenticated user
pub async fn handler(api_token: ApiToken) -> Response {
    let resp_msg = serde_json::json!({"account_id": api_token.subject()});
    (StatusCode::OK, Json(resp_msg)).into_response()
}
