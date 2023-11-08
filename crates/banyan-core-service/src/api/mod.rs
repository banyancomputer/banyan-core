use axum::Router;
use tower_http::cors::CorsLayer;

mod auth;
mod blocks;
mod buckets;
mod users;
pub mod models;

use crate::app::AppState;

pub fn router(state: AppState) -> Router<AppState> {
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
        .with_state(state)
        .layer(cors_layer)
}
