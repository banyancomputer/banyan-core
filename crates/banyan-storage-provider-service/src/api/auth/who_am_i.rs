use axum::extract::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::extractors::AuthenticatedClient;

/// Return the account id of the currently authenticated user
pub async fn handler(identity: AuthenticatedClient) -> Response {
    let resp_msg = serde_json::json!({
        "consumed_storage": identity.consumed_storage(),
        "platform_id": identity.platform_id(),
        "remaining_storage": identity.remaining_storage(),
    });

    (StatusCode::OK, Json(resp_msg)).into_response()
}
