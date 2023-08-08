use axum::routing::{get, post};
use axum::Router;

mod car_buffer;
mod error;
mod handlers;
mod metadata;
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
        .nest("/:bucket_id/metadata", metadata::router(state.clone()))
        .with_state(state)
}
