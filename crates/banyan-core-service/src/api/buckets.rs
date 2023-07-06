use axum::routing::get;
use axum::Router;

mod error;
mod handlers;
mod requests;
mod responses;

pub use error::Error as BucketError;

pub fn router() -> Router {
    Router::new()
        .route("/", get(handlers::index).post(handlers::create))
        .route("/:bucket_id", get(handlers::show))
}
