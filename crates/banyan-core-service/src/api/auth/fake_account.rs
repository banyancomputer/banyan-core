use axum::routing::{post};
use axum::Router;

mod handlers;
mod requests;
mod responses;

use crate::app_state::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(handlers::create))
        .with_state(state)
}