use axum::routing::{get, post};
use axum::Router;

mod error;
mod handlers;
mod requests;
mod responses;

pub use error::Error as BucketError;

pub fn router() -> Router {
    Router::new()
        .route("/", get(handlers::index).post(handlers::create))
        .route("/:bucket_id", get(handlers::show).delete(handlers::destroy))
        .route("/:bucket_id/publish", post(handlers::publish_metadata))
}
