use axum::routing::get;
use axum::Router;

mod error;
mod handlers;
mod requests;
mod responses;

mod keys;
mod metadata;
mod snapshots;

pub use error::Error as BucketError;

use crate::app_state::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(handlers::read_all).post(handlers::create))
        .route("/usage", get(handlers::get_total_usage))
        .route("/usage_limit", get(handlers::get_usage_limit))
        .route("/:bucket_id/usage", get(handlers::get_usage))
        .route("/:bucket_id", get(handlers::read).delete(handlers::delete))
        .nest("/:bucket_id/keys", keys::router(state.clone()))
        .nest("/:bucket_id/metadata", metadata::router(state.clone()))
        .nest("/:bucket_id/snapshots", snapshots::router(state.clone()))
        .with_state(state)
}
