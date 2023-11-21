use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::CorsLayer;

mod block_retrieval;
mod client_grant;
mod prune_blocks;
mod upload;

use crate::app::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    let cors_layer = CorsLayer::very_permissive();

    Router::new()
        .route("/blocks/:block_id", get(block_retrieval::handler))
        .route("/client_grant", post(client_grant::handler))
        .route("/upload", post(upload::handler))
        .route("/core/prune", post(prune_blocks::handler))
        .layer(cors_layer)
        .with_state(state)
}
