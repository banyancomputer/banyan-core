use axum::routing::post;
use axum::Router;

mod location;

use crate::app_state::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/location", post(location::handler))
        .with_state(state)
}
