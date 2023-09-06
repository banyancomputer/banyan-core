use axum::Router;
use tower_http::cors::CorsLayer;

use crate::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    let cors_layer = CorsLayer::very_permissive();

    Router::new().layer(cors_layer).with_state(state)
}
