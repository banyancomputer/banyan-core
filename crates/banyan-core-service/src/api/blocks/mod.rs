use axum::routing::post;
use axum::Router;
use serde::de::StdError;

mod locate;

use crate::app::AppState;

pub fn router<B>(state: AppState) -> Router<AppState,B>
    where
        B: axum::body::HttpBody + Send + 'static,
        B::Data: Send,
        Box<dyn StdError + Send + Sync + 'static>: From<B::Error>,
{
    Router::new()
        .route("/locate", post(locate::handler))
        .with_state(state)
}
