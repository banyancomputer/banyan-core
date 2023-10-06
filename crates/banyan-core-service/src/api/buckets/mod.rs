use axum::routing::{delete, get, post};
use axum::Router;

mod all_buckets;
mod create_bucket;
pub mod common;
mod current_total_usage;
mod current_total_usage_limit;
mod single_bucket;

//mod keys;
//mod metadata;
//mod snapshots;

use crate::app::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(all_buckets::handler).post(create_bucket::handler),
        )
        .route("/:bucket_id", get(single_bucket::handler))
        .route("/usage", get(current_total_usage::handler))
        .route("/usage_limit", get(current_total_usage_limit::handler))
        //.route("/:bucket_id", delete(handlers::delete))
        //.route("/:bucket_id/usage", get(handlers::get_usage))
        //.nest("/:bucket_id/keys", keys::router(state.clone()))
        //.nest("/:bucket_id/metadata", metadata::router(state.clone()))
        //.nest("/:bucket_id/snapshots", snapshots::router(state.clone()))
        .with_state(state)
}
