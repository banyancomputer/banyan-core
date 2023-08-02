use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, Router};
use http::header::{ACCEPT, AUTHORIZATION, ORIGIN};
use http::Method;
use serde::Serialize;
use tower_http::cors::{Any, CorsLayer};

use crate::app_state::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    // todo: Ideally this would have a wrapper method to allow per route method configuration or
    // even better something that inspected the route matches and applied the correct method config
    // for that path...
    let cors_layer = CorsLayer::new()
        .allow_methods(vec![Method::GET, Method::DELETE, Method::POST, Method::PUT])
        .allow_headers(vec![AUTHORIZATION, ACCEPT, ORIGIN])
        // todo: add domain as a config option and make this configurable
        .allow_origin(Any)
        .allow_credentials(false);

    Router::new()
        .with_state(state)
        .layer(cors_layer)
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub errors: Vec<String>,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(self)).into_response()
    }
}