use axum::routing::get;
use axum::Router;

mod handlers;
mod responses;

use crate::app_state::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::read))
        .with_state(state)
}
