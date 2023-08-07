use axum::routing::{get, post};
use axum::Router;

//mod error;
mod handlers;
//mod models;
//mod requests;
//mod responses;

use crate::app_state::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::index))
        .route(
            "/:metadata_id",
            get(handlers::show).delete(handlers::destroy),
        )
        .route("/:metadata_id/download", get(handlers::download))
        .route("/:metadata_id/snapshot", post(handlers::snapshot))
        .with_state(state)
}
