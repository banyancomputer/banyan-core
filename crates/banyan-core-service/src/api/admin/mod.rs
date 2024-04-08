use std::error::Error;

use axum::body::HttpBody;
use axum::routing::get;
use axum::Router;

mod all_deals;
mod storage_host;
mod users;

use crate::app::AppState;

pub fn router<B>(state: AppState) -> Router<AppState, B>
where
    B: HttpBody + Send + 'static,
    B::Data: Send + 'static,
    Box<dyn Error + Send + Sync + 'static>: From<B::Error>,
{
    Router::new()
        .route("/deals", get(all_deals::handler))
        .nest("/users", users::router(state.clone()))
        .nest("/providers", storage_host::router(state.clone()))
        .with_state(state)
}
