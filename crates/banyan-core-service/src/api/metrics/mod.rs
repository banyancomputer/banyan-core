mod traffic;

use std::error::Error;

use axum::body::HttpBody;
use axum::routing::post;
use axum::Router;

use crate::app::AppState;
pub fn router<B>(state: AppState) -> Router<AppState, B>
where
    B: HttpBody + Send + 'static,
    B::Data: Send + 'static,
    Box<dyn Error + Send + Sync + 'static>: From<B::Error>,
{
    Router::new()
        .route("/traffic", post(traffic::handler))
        .with_state(state)
}
