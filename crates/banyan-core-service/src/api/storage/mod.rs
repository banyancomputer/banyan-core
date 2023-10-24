mod finalize_upload;

use axum::routing::post;
use axum::Router;

use crate::app::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/:metadata_id", post(finalize_upload::handler))
        .with_state(state)
}
