mod delete_blocks;
mod distribute;
use std::error::Error;

use axum::body::HttpBody;
use axum::routing::{delete, post};
use axum::Router;

use crate::app::AppState;

pub fn router<B>(state: AppState) -> Router<AppState, B>
where
    B: HttpBody + Send + 'static,
    B::Data: Send + 'static,
    Box<dyn Error + Send + Sync + 'static>: From<B::Error>,
    bytes::Bytes: From<<B as HttpBody>::Data>,
{
    Router::new()
        .route("/blocks", delete(delete_blocks::handler))
        .route("/distribute", post(distribute::handler))
        .with_state(state)
}
