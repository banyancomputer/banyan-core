use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

pub async fn server_error_handler(error: tower::BoxError) -> Response {
    let mut errors = vec![error.to_string()];
    let mut source = error.source();

    while let Some(inner_err) = source {
        errors.push(inner_err.to_string());
        source = inner_err.source();
    }

    tracing::error!(errors = ?errors, "unhandled error");

    // Some of our errors have specific error handling requirements
    if error.is::<tower::timeout::error::Elapsed>() {
        let msg = serde_json::json!({"status": "error", "message": "request timed out"});
        return (StatusCode::REQUEST_TIMEOUT, Json(msg)).into_response();
    }

    if error.is::<tower::load_shed::error::Overloaded>() {
        let msg = serde_json::json!({"status": "error", "message": "service overloaded"});
        return (StatusCode::SERVICE_UNAVAILABLE, Json(msg)).into_response();
    }

    let msg = serde_json::json!({"status": "error", "message": "unknown server error"});
    (StatusCode::INTERNAL_SERVER_ERROR, Json(msg)).into_response()
}

pub async fn not_found_handler() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({"status": "not found"})),
    ).into_response()
}
