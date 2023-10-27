mod mailgun;
mod prune_blocks;

use axum::routing::post;
use axum::Router;
use tower_http::cors::CorsLayer;

use crate::app::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    // TODO: does this need to be here?
    // TODO: Find the right cors config for this
    let cors_layer = CorsLayer::very_permissive();

    Router::new()
        .route("/mailgun", post(mailgun::handler))
        .route("/storage/prune", post(prune_blocks::handler))
        .with_state(state)
        .layer(cors_layer)
}
