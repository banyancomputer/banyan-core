use std::error::Error;

use axum::body::HttpBody;
use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::CorsLayer;

mod read;
mod write;

use crate::app::AppState;

pub fn router<B>(state: AppState) -> Router<AppState, B>
where
    B: HttpBody + Send + 'static,
    B::Data: Send,
    bytes::Bytes: From<B::Data>,
    Box<dyn Error + Send + Sync + 'static>: From<B::Error>,
{
    let cors_layer = CorsLayer::very_permissive();

    Router::new()
        .route("/blocks/:block_id", get(read::handler))
        .route("/blocks", post(write::handler))
        .layer(cors_layer)
        .with_state(state)
}
