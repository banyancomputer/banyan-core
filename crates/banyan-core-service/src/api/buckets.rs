use axum::routing::get;
use axum::Router;

mod error;
mod handlers;
mod responses;

pub use error::Error as BucketError;

pub fn router() -> Router {
    Router::new()
        .route("/:bucket_id", get(handlers::show))
}
