use axum::routing::get;
use std::error::Error;

use axum::body::HttpBody;
use axum::Router;

mod shared_file;

use crate::app::AppState;

pub fn router<B>(state: AppState) -> Router<AppState, B>
where
    B: HttpBody + Send + 'static,
    B::Data: Send + 'static,
    Box<dyn Error + Send + Sync + 'static>: From<B::Error>,
    bytes::Bytes: From<<B as HttpBody>::Data>,
{
    Router::new()
        .route("/", get(shared_file::handler))
        .with_state(state)
}
