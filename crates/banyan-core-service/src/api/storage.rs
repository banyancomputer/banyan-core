use axum::routing::{delete, get, post};
use axum::Router;

mod error;
mod handlers;
mod models;
mod requests;
mod responses;

pub use error::Error as StorageError;

use crate::app_state::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::current_authorizations).post(handlers::create_authorization))
        .route("/:auth_id", delete(handlers::revoke_authorization))
        .with_state(state)
}
