mod all_metadata;
mod current_metadata;
mod delete_metadata;
mod single_metadata;

mod pull_metadata;
mod push_metadata;

pub use push_metadata::STORAGE_TICKET_DURATION;

use axum::routing::{get, post};
use axum::Router;

use crate::app::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(push_metadata::handler).get(all_metadata::handler))
        .route("/current", get(current_metadata::handler))
        .route(
            "/:metadata_id",
            get(single_metadata::handler).delete(delete_metadata::handler),
        )
        .route("/:metadata_id/pull", get(pull_metadata::handler))
        .with_state(state)
}
