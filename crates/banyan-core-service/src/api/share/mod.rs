use axum::routing::get;
use axum::Router;

use crate::app::AppState;

mod shared_file;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(shared_file::handler))
        .with_state(state)
}
