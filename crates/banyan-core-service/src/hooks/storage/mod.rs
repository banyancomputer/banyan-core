mod prune_blocks;
mod report_upload;

use axum::routing::post;
use axum::Router;
use tower_http::cors::CorsLayer;
use std::error::Error;

use crate::app::AppState;

pub fn router<B>(state: AppState) -> Router<AppState,B>
where
    B: axum::body::HttpBody + Send + 'static,
    B::Data: Send + 'static,
    B::Error: Error + Send + Sync + 'static,
{
    // TODO: Find the right cors config for this[
    let cors_layer = CorsLayer::very_permissive();

    Router::new()
        .route("/report/:metadata_id", post(report_upload::handler))
        .route("/prune", post(prune_blocks::handler))
        .with_state(state)
        .layer(cors_layer)
}
