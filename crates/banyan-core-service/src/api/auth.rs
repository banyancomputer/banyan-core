use axum::routing::{get, post};
use axum::Router;

mod error;
mod handlers;
mod models;
mod requests;
mod responses;

pub use error::Error as AuthError;

use crate::app_state::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/create_fake_account", get(handlers::create_fake_account))
        .route("/register_device_key", post(handlers::register_device_key))
        .route("/whoami", get(handlers::whoami))
        .with_state(state)
}
