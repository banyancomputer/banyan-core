#![allow(dead_code)]
use axum::routing::post;
use axum::Router;

mod error;
mod handler;
mod request;

pub use error::MailgunHookError;

use crate::app_state::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(handler::handle))
        .with_state(state)
}
