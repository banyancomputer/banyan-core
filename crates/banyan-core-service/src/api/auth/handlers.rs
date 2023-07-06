use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub async fn fake_token() -> Response {
    (StatusCode::OK, "JWT").into_response()
}
