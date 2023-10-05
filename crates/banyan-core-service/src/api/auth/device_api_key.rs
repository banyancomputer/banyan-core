use axum::routing::{delete, get, post};
use axum::Router;

mod handlers;
mod requests;
mod responses;

use crate::app::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(handlers::create))
        .route("/:id", get(handlers::read))
        .route("/", get(handlers::read_all))
        .route("/:id", delete(handlers::delete))
        .with_state(state)
}
