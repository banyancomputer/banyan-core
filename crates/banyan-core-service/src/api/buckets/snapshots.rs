use axum::routing::get;
use axum::Router;

mod handlers;
mod requests;
mod responses;

use crate::app_state::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::read_all).post(handlers::create))
        .route("/:snapshot_id", get(handlers::read))
        .route("/:snapshot_id/pull", get(handlers::pull))
        .with_state(state)
}
