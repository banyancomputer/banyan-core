use axum::Router;
use serde::de::StdError;
use tower_http::cors::CorsLayer;

mod auth;
mod blocks;
mod buckets;
pub mod models;
mod users;

use crate::app::AppState;

pub fn router<B>(state: AppState) -> Router<AppState,B>
where
    B: axum::body::HttpBody + Send + 'static,
    B::Data:  Send + 'static,
    Box<dyn StdError + Send + Sync + 'static>: From<B::Error>,
{
    // TODO: Ideally this would have a wrapper method to allow per route method configuration or
    // even better something that inspected the route matches and applied the correct method config
    // for that path...
    // TODO: Find the right cors config for this
    let cors_layer = CorsLayer::very_permissive();

    Router::new()
        .nest("/auth", auth::router(state.clone()))
        .nest("/users", users::router(state.clone()))
        .nest("/blocks", blocks::router(state.clone()))
        .nest("/buckets", buckets::router(state.clone()))
        .layer(cors_layer)
        .with_state(state)
}
