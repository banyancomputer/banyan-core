use std::error::Error;

use axum::body::HttpBody;
use axum::routing::get;
use axum::Router;

use crate::app::AppState;

mod who_am_i;

pub fn router<B>(state: AppState) -> Router<AppState, B>
where
    B: HttpBody + Send + 'static,
    B::Data: Send + 'static,
    Box<dyn Error + Send + Sync + 'static>: From<B::Error>,
{
    Router::new()
        .route("/who_am_i", get(who_am_i::handler))
        .with_state(state)
}
