use axum::routing::{delete, get, post};
use axum::Router;

mod handlers;
mod requests;
pub mod responses;

use crate::app_state::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(handlers::create))
        .route("/", get(handlers::read_all))
        .route("/:bucket_key_id", get(handlers::read))
        .route("/:bucket_key_id", delete(handlers::delete))
        .with_state(state)
}
