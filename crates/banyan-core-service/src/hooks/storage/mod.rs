mod complete_distribution;
mod prune_blocks;
mod report_health;
mod report_upload;

use std::error::Error;

use axum::body::HttpBody;
use axum::routing::post;
use axum::Router;
use tower_http::cors::CorsLayer;

use crate::app::AppState;

pub fn router<B>(state: AppState) -> Router<AppState, B>
where
    B: HttpBody + Send + 'static,
    B::Data: Send + 'static,
    B::Error: Error + Send + Sync + 'static,
{
    // TODO: Find the right cors config for this
    let cors_layer = CorsLayer::very_permissive();

    Router::new()
        .route("/report/:metadata_id", post(report_upload::handler))
        .route(
            "/distribution/:metadata_id",
            post(complete_distribution::handler),
        )
        .route("/prune", post(prune_blocks::handler))
        .route("/report/health", post(report_health::handler))
        .layer(cors_layer)
        .with_state(state)
}
