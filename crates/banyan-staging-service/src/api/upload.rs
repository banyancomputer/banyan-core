use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::extractors::UploadStore;

pub async fn handler(_store: UploadStore) -> Response {
    (StatusCode::NO_CONTENT, ()).into_response()
}
