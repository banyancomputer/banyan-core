use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

pub async fn handler() -> Response {
    let err_msg = serde_json::json!({"msg": "not yet implmented"});
    (StatusCode::NOT_IMPLEMENTED, Json(err_msg)).into_response()
}
