use axum::routing::{delete, get, post};
use axum::Router;

//mod error;
//mod handlers;
//mod requests;
//mod responses;

//mod keys;
//mod metadata;
//mod snapshots;


mod create_bucket;

use crate::app::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", post(create_bucket::handler))
        //.route("/", get(handlers::read_all))
        //.route("/usage", get(handlers::get_total_usage))
        //.route("/usage_limit", get(handlers::get_usage_limit))
        //.route("/:bucket_id/usage", get(handlers::get_usage))
        //.route("/:bucket_id", get(handlers::read))
        //.route("/:bucket_id", delete(handlers::delete))
        //.nest("/:bucket_id/keys", keys::router(state.clone()))
        //.nest("/:bucket_id/metadata", metadata::router(state.clone()))
        //.nest("/:bucket_id/snapshots", snapshots::router(state.clone()))
        .with_state(state)
}
