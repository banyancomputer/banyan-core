use axum::routing::{delete, get, post};
use axum::Router;

mod create_bucket;
mod current_total_usage;
mod current_total_usage_limit;
mod read_all_buckets;

//mod keys;
//mod metadata;
//mod snapshots;

use crate::app::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(read_all_buckets::handler).post(create_bucket::handler),
        )
        .route("/usage", get(current_total_usage::handler))
        .route("/usage_limit", get(current_total_usage_limit::handler))
        //.route("/:bucket_id/usage", get(handlers::get_usage))
        //.route("/:bucket_id", get(handlers::read))
        //.route("/:bucket_id", delete(handlers::delete))
        //.nest("/:bucket_id/keys", keys::router(state.clone()))
        //.nest("/:bucket_id/metadata", metadata::router(state.clone()))
        //.nest("/:bucket_id/snapshots", snapshots::router(state.clone()))
        .with_state(state)
}
