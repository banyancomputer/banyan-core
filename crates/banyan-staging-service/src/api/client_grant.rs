use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::extractors::StorageGrant;

pub async fn handler(_grant: StorageGrant) -> Response {
    (StatusCode::NO_CONTENT, ()).into_response()
}
