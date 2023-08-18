use axum::routing::{get, post};
use axum::Router;

mod handlers;
mod requests;
mod responses;

use crate::app_state::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(handlers::create).get(handlers::read_all))
        .route("/:snapshot_id",get(handlers::read))
        .route("/:snapshot_id/pull", get(handlers::pull))
        .with_state(state)
}
