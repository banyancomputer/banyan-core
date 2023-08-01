use axum::routing::{get, post};
use axum::Router;

mod error;
mod handlers;
mod header_buffer;
mod models;
mod requests;
mod responses;

pub use error::Error as BucketError;

use crate::app_state::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::index).post(handlers::create))
        .route("/:bucket_id", get(handlers::show).delete(handlers::destroy))
        .route("/:bucket_id/publish", post(handlers::publish_metadata))
        .with_state(state)
}
