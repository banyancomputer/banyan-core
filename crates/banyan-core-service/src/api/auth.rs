use axum::routing::get;
use axum::Router;

mod error;
mod handlers;
mod responses;

pub use error::Error as AuthError;

use crate::app_state::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/fake_token", get(handlers::fake_token))
        .with_state(state)
}
