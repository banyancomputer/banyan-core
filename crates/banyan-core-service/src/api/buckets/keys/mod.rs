use std::error::Error;

use axum::body::HttpBody;
use axum::routing::{get, post};
use axum::Router;

mod all_bucket_keys;
mod create_bucket_key;
mod delete_bucket_key;
mod single_bucket_key;

use crate::app::AppState;

pub fn router<B>(state: AppState) -> Router<AppState, B>
where
    B: HttpBody + Send + 'static,
    B::Data: Send,
    Box<dyn Error + Send + Sync + 'static>: From<B::Error>,
{
    Router::new()
        .route(
            "/",
            get(all_bucket_keys::handler).post(create_bucket_key::handler),
        )
        .route(
            "/:bucket_key_id",
            get(single_bucket_key::handler).delete(delete_bucket_key::handler),
        )
        .route("/:bucket_key_id/reject", post(delete_bucket_key::handler))
        .with_state(state)
}
