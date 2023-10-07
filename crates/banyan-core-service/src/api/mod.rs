use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, Router};
use serde::Serialize;

mod auth;
mod buckets;
pub mod models;
mod storage;

use crate::app::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    // TODO: Ideally this would have a wrapper method to allow per route method configuration or
    // even better something that inspected the route matches and applied the correct method config
    // for that path...
    // TODO: Find the right cors config for this
    let cors_layer = tower_http::cors::CorsLayer::very_permissive();

    Router::new()
        .nest("/auth", auth::router(state.clone()))
        .nest("/buckets", buckets::router(state.clone()))
        .nest("/storage", storage::router(state.clone()))
        .with_state(state)
        .layer(cors_layer)
}
