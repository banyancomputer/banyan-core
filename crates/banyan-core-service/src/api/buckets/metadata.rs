use axum::routing::{get, post};
use axum::Router;

mod handlers;
mod requests;
mod responses;

use crate::app_state::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(handlers::push).get(handlers::read_all))
        .route("/current", get(handlers::read_current))
        .route(
            "/:metadata_id",
            get(handlers::read).delete(handlers::delete),
        )
        .route("/:metadata_id/pull", get(handlers::pull))
        .with_state(state)
}
