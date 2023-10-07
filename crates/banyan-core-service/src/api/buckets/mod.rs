use axum::routing::{delete, get, post};
use axum::Router;

//mod keys;
mod metadata;
//mod snapshots;

mod all_buckets;
mod bucket_usage;
mod create_bucket;
mod current_total_usage;
mod current_total_usage_limit;
mod delete_bucket;
mod single_bucket;

use crate::app::AppState;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/:bucket_id", get(single_bucket::handler).delete(delete_bucket::handler))
        //.nest("/:bucket_id/keys", keys::router(state.clone()))
        .nest("/:bucket_id/metadata", metadata::router(state.clone()))
        //.nest("/:bucket_id/snapshots", snapshots::router(state.clone()))
        .route("/:bucket_id/usage", get(bucket_usage::handler))
        .route("/usage", get(current_total_usage::handler))
        .route("/usage_limit", get(current_total_usage_limit::handler))
        .route(
            "/",
            get(all_buckets::handler).post(create_bucket::handler),
        )
        .with_state(state)
}
