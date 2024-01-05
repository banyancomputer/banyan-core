use std::error::Error;

use axum::body::HttpBody;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use http::StatusCode;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

use crate::app::AppState;
use crate::database::models::{Deal, DealState};

pub fn router<B>(state: AppState) -> Router<AppState, B>
where
    B: HttpBody + Send + 'static,
    B::Data: Send + 'static,
{
    // TODO: Find the right cors config for this
    let cors_layer = CorsLayer::very_permissive();

    Router::new()
        .route("/providers/:provider_id", get(get_provider_handler))
        .route("/providers", get(all_providers_handler))
        .layer(cors_layer)
        .with_state(state)
}

pub async fn all_providers_handler() -> Response {
    (StatusCode::OK, Json(())).into_response()
}

pub async fn get_provider_handler(Path(provider_id): Path<Uuid>) -> Response {
    (StatusCode::OK, Json(())).into_response()
}
