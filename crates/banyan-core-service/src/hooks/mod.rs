mod mailgun;

use axum::Router;
use axum::routing::post;
use tower_http::cors::CorsLayer;

use crate::app::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    // TODO: Find the right cors config for this
    let cors_layer = CorsLayer::very_permissive();

    Router::new()
        .route("/mailgun", post(mailgun::handler))
        .with_state(state)
        .layer(cors_layer)
}
