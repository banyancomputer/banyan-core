use std::error::Error;

use axum::body::HttpBody;
use axum::routing::get;
use axum::Router;

mod all_storage_hosts;
mod create_storage_host;

use crate::app::AppState;

pub fn router<B>(state: AppState) -> Router<AppState, B>
where
    B: HttpBody + Send + 'static,
    B::Data: Send + 'static,
    Box<dyn Error + Send + Sync + 'static>: From<B::Error>,
{
    Router::new()
        .route(
            "/",
            get(all_storage_hosts::handler).post(create_storage_host::handler),
        )
        .with_state(state)
}
