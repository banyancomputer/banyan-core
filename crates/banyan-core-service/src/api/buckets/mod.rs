mod metadata;
mod snapshots;

mod all_bucket_access;
mod all_buckets;
mod create_bucket;
mod delete_bucket;
mod single_bucket;
mod single_bucket_access;
mod update_bucket;

mod bucket_usage;
mod current_total_usage;
mod current_total_usage_limit;

mod authorization_grants;

use std::error::Error;

use axum::body::HttpBody;
use axum::routing::get;
use axum::Router;

use crate::app::AppState;

pub fn router<B>(state: AppState) -> Router<AppState, B>
where
    B: HttpBody + Send + 'static,
    B::Data: Send,
    Box<dyn Error + Send + Sync + 'static>: From<B::Error>,
    bytes::Bytes: From<<B as HttpBody>::Data>,
{
    Router::new()
        .route(
            "/:bucket_id",
            get(single_bucket::handler)
                .put(update_bucket::handler)
                .delete(delete_bucket::handler),
        )
        .route("/:bucket_id/access", get(all_bucket_access::handler))
        .route(
            "/:bucket_id/access/:user_key_id",
            get(single_bucket_access::handler),
        )
        .nest("/:bucket_id/metadata", metadata::router(state.clone()))
        .nest("/:bucket_id/snapshots", snapshots::router(state.clone()))
        .route("/:bucket_id/usage", get(bucket_usage::handler))
        .route(
            "/:bucket_id/authorization_grants",
            get(authorization_grants::handler),
        )
        .route("/usage", get(current_total_usage::handler))
        .route("/usage_limit", get(current_total_usage_limit::handler))
        .route("/", get(all_buckets::handler).post(create_bucket::handler))
        .with_state(state)
}
