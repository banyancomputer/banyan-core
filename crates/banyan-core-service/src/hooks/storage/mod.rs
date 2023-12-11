mod prune_blocks;
mod report_upload;

use axum::body::HttpBody;
use axum::routing::post;
use axum::Router;
use std::error::Error;
use tower_http::cors::CorsLayer;

use crate::app::AppState;

pub fn router<B>(state: AppState) -> Router<AppState, B>
where
    B: HttpBody + Send + 'static,
    B::Data: Send + 'static,
    B::Error: Error + Send + Sync + 'static,
{
    let cors_layer = CorsLayer::very_permissive();

    Router::new()
        .route("/report/:metadata_id", post(report_upload::handler))
        .route("/prune", post(prune_blocks::handler))
        .layer(cors_layer)
        .with_state(state)
}
