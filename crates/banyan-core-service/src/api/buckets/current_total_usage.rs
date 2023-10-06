use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::extractors::ApiToken;

pub async fn handler(_api_token: ApiToken) -> Response {
    todo!()
}
