use std::error::Error;

use axum::body::HttpBody;
use axum::routing::post;
use axum::Router;

mod locate;
mod provider_locate;

use crate::app::AppState;

pub fn router<B>(state: AppState) -> Router<AppState, B>
where
    B: HttpBody + Send + 'static,
    B::Data: Send,
    Box<dyn Error + Send + Sync + 'static>: From<B::Error>,
{
    Router::new()
        .route("/locate", post(locate::handler))
        .route("/provider_locate", post(provider_locate::handler))
        .with_state(state)
}
