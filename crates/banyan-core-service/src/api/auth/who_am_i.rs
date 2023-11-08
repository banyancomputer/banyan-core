use axum::extract::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::extractors::UserIdentity;

/// Return the account id of the currently authenticated user
pub async fn handler(user_identity: UserIdentity) -> Response {
    let user_id = user_identity.id().to_string();
    let resp_msg = serde_json::json!({"user_id": user_id});
    (StatusCode::OK, Json(resp_msg)).into_response()
}
