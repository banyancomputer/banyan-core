use axum::routing::{post, get};
use axum::Router;

mod handlers;
mod requests;
mod responses;

use crate::app_state::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(handlers::push).get(handlers::read_all))
        .route("/:bucket_metadata_id", get(handlers::read).delete(handlers::delete))
        .route("/:bucket_metadata_id/pull", get(handlers::pull))
        .route("/:bucket_metadata_id/snapshot", get(handlers::snapshot))
        .with_state(state)
}
