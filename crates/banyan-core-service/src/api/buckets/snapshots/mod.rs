use axum::routing::{get, put, post};
use axum::Router;

mod all_snapshots;
mod create_snapshot;
mod restore_snapshot;

use crate::app::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(all_snapshots::handler))
        .route("/:metadata_id", post(create_snapshot::handler))
        .route("/:snapshot_id/restore", put(restore_snapshot::handler))
        .with_state(state)
}
