use std::fmt::Display;

use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;
use rand::Rng;
use serde::Serialize;
use time::ext::NumericalDuration;
use tower_http::cors::CorsLayer;

mod alerts;
mod block_retrieval;
mod client_grant;
mod deals;
mod metrics;
mod prune_blocks;
mod upload;

use crate::app::AppState;

const CURRENCY_MULTIPLIER: usize = 10_000;

const PRICE_PER_TIB: usize = 2 * CURRENCY_MULTIPLIER;

pub fn router(state: AppState) -> Router<AppState> {
    let cors_layer = CorsLayer::very_permissive();

    Router::new()
        // TODO: Should we place these behind a new prefix?
        // Client Storage API routes
        .route("/blocks/:block_id", get(block_retrieval::handler))
        .route("/client_grant", post(client_grant::handler))
        .route("/upload", post(upload::handler))
        .route("/upload/new", post(upload::new::handler))
        .route("/upload/block", post(upload::block::handler))
        .route("/core/prune", post(prune_blocks::handler))
        .nest("/alerts", alerts::router(state.clone()))
        .nest("/deals", deals::router(state.clone()))
        .nest("/metrics", metrics::router(state.clone()))
        .layer(cors_layer)
        .with_state(state)
}
