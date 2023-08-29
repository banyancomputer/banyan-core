use axum::routing::{get, post};
use axum::Router;

mod handlers;
mod requests;
pub mod responses;

use crate::app_state::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::read_all).post(handlers::create))
        .route(
            "/:bucket_key_id",
            get(handlers::read).delete(handlers::delete),
        )
        .with_state(state)
}
