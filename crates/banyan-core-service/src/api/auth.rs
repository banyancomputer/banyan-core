use axum::routing::{get, post};
use axum::Router;

mod error;
mod handlers;
mod requests;
mod responses;

pub use error::Error as AuthError;

use crate::app_state::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/fake_register", get(handlers::fake_register))
        .route("/register_device_key", post(handlers::register_device_key))
        .with_state(state)
}
