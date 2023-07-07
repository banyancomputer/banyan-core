use axum::routing::get;
use axum::Router;

mod error;
mod handlers;
mod responses;

pub use error::Error as AuthError;

pub fn router() -> Router {
    Router::new().route("/fake_token", get(handlers::fake_token))
}
