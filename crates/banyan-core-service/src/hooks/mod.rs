mod mailgun;
mod storage;

use storage::router as storage_router;

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
        .nest("/storage", storage_router(state.clone()))
        .with_state(state)
        .layer(cors_layer)
}
