use axum::extract::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::extractors::{Identity, UserIdentity};

/// Return the account id of the currently authenticated user
pub async fn handler(user_id: UserIdentity) -> Response {
    let resp_msg = serde_json::json!({"user_id": user_id.user_id()});
    (StatusCode::OK, Json(resp_msg)).into_response()
}
