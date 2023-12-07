use axum::response::{IntoResponse, Response};
use axum::Json;
use http::StatusCode;

use crate::app::Version;

pub async fn handler() -> Response {
    (StatusCode::OK, Json(Version::new())).into_response()
}