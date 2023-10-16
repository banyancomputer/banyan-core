use axum::routing::{delete, get, post};
use axum::Router;

mod handlers;
mod requests;
mod responses;

use crate::app_state::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(handlers::create))
        .route("/:id", get(handlers::read))
        .route("/", get(handlers::read_all))
        .route("/:id", delete(handlers::delete))
        .route("/start_regwait/:fingerprint", get(handlers::start_regwait))
        .route("/end_regwait/:fingerprint", get(handlers::end_regwait))
        .with_state(state)
}
