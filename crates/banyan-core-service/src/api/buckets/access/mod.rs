use std::error::Error;

use axum::body::HttpBody;
use axum::routing::{get, post};
use axum::Router;

mod all_bucket_access;
mod revoke_bucket_access;
mod single_bucket_access;

use crate::app::AppState;

pub fn router<B>(state: AppState) -> Router<AppState, B>
where
    B: HttpBody + Send + 'static,
    B::Data: Send,
    Box<dyn Error + Send + Sync + 'static>: From<B::Error>,
{
    Router::new()
        .route("/", get(all_bucket_access::handler))
        .route(
            "/:api_key",
            get(single_bucket_access::handler).delete(revoke_bucket_access::handler),
        )
        .route("/:api_key/reject", post(revoke_bucket_access::handler))
        .with_state(state)
}
