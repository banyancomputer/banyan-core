use std::error::Error;

use axum::body::HttpBody;
use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::CorsLayer;

mod client_grant;
mod prune_blocks;
mod read_block;
mod upload;

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
        .route("/blocks", post(upload::write_block::handler))
        .route("/blocks/:block_id", get(read_block::handler))
        .route("/client_grant", post(client_grant::handler))
        .route("/upload", post(upload::handler))
        .route("/core/prune", post(prune_blocks::handler))
        .layer(cors_layer)
        .with_state(state)
}
