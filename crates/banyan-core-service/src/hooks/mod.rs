mod mailgun;
mod storage;

use std::error::Error;
use axum::body::HttpBody;
use axum::routing::post;
use axum::Router;
use tower_http::cors::CorsLayer;

use crate::app::AppState;

pub fn router<B>(state: AppState) -> Router<AppState,B>
where
B: axum::body::HttpBody + Send + 'static,
B::Data: Send + 'static,
B::Error: Into<Box<dyn std::error::Error + Send + Sync>> + std::error::Error + Sync + Send + 'static, 


{
    // TODO: Find the right cors config for this
    let cors_layer = CorsLayer::very_permissive();

    Router::new()
        .route("/mailgun", post(mailgun::handler))
        .nest("/storage", storage::router(state.clone()))
        .with_state(state)
        .layer(cors_layer)
}
