mod all_metadata;
mod current_metadata;
mod delete_metadata;
mod pull_metadata;
mod push_metadata;
mod single_metadata;

use std::error::Error;

use axum::body::HttpBody;
use axum::routing::{get, post};
use axum::Router;

pub use self::push_metadata::STORAGE_TICKET_DURATION;
use super::snapshots::create_snapshot;
use crate::app::AppState;

pub fn router<B>(state: AppState) -> Router<AppState, B>
where
    B: HttpBody + Send + 'static,
    B::Data: Send,
    Box<dyn Error + Send + Sync + 'static>: From<B::Error>,
    bytes::Bytes: From<<B as HttpBody>::Data>,
{
    Router::new()
        .route("/", post(push_metadata::handler).get(all_metadata::handler))
        .route("/current", get(current_metadata::handler))
        .route(
            "/:metadata_id",
            get(single_metadata::handler).delete(delete_metadata::handler),
        )
        .route("/:metadata_id/pull", get(pull_metadata::handler))
        .route("/:metadata_id/snapshot", post(create_snapshot::handler))
        .with_state(state)
}
