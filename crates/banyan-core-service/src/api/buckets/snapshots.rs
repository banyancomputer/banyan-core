use axum::routing::{get, put};
use axum::Router;

mod handlers;
mod requests;
mod responses;

use crate::app_state::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::read_all).post(handlers::create))
        .route("/:snapshot_id/restore", put(handlers::restore))
        .with_state(state)
}
