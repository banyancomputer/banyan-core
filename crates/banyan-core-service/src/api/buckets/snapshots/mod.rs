use axum::body::HttpBody;
use axum::routing::{get, put};
use axum::Router;

mod all_snapshots;
mod restore_snapshot;
mod single_snapshot;

use crate::app::AppState;

pub fn router<B>(state: AppState) -> Router<AppState, B>
where
    B: HttpBody + Send + 'static,
{
    Router::new()
        .route("/", get(all_snapshots::handler))
        .route("/:snapshot_id", get(single_snapshot::handler))
        .route("/:snapshot_id/restore", put(restore_snapshot::handler))
        .with_state(state)
}
