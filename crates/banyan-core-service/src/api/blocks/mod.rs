use axum::routing::post;
use axum::Router;

mod locate;

use crate::app_state::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/locate", post(locate::handler))
        .with_state(state)
}
