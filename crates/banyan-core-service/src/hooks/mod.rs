mod mailgun;
mod storage;

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
    B::Error: Into<Box<dyn Error + Send + Sync>> + Error + Sync + Send + 'static,
{
    // TODO: Find the right cors config for this
    let cors_layer = CorsLayer::very_permissive();

    Router::new()
        .route("/mailgun", post(mailgun::handler))
        .nest("/storage", storage::router(state.clone()))
        .layer(cors_layer)
        .with_state(state)
}
