use axum::routing::{get, put};
use axum::Router;

mod all_snapshots;

//mod handlers;
//mod requests;
//mod responses;

use crate::app::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(all_snapshots::handler))
        //.route("/", get(handlers::read_all).post(handlers::create))
        //.route("/:snapshot_id/restore", put(handlers::restore))
        .with_state(state)
}
