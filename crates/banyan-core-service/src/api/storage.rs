use axum::routing::post;
use axum::Router;

mod error;
mod handlers;
mod requests;
mod responses;

pub use error::Error as StorageError;

use crate::app_state::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/:metadata_id", post(handlers::finalize_upload))
        .with_state(state)
}
