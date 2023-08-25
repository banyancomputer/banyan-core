use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::extractors::AuthenticatedClient;
use crate::extractors::UploadStore;

pub async fn handler(_client: AuthenticatedClient, _store: UploadStore) -> Response {
    (StatusCode::NO_CONTENT, ()).into_response()
}
