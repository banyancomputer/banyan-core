use axum::routing::post;
use axum::Router;

use crate::app::AppState;

mod finalize_upload;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/:metadata_id", post(finalize_upload::handler))
        .with_state(state)
}
