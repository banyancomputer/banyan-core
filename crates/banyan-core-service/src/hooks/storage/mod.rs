mod prune_blocks;

use axum::routing::post;
use axum::Router;

use crate::app::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/prune", post(prune_blocks::handler))
        .with_state(state)
}
