use std::error::Error;

use axum::body::HttpBody;
use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::CorsLayer;

mod auth;
mod block_retrieval;
mod client_grant;
mod hooks;
mod prune_blocks;
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
        .nest("/auth", auth::router(state.clone()))
        .nest("/hooks", hooks::router(state.clone()))
        .route("/blocks/:block_id", get(block_retrieval::handler))
        .route("/client_grant", post(client_grant::handler))
        .route("/upload", post(upload::handler))
        .route("/upload/new", post(upload::new::handler))
        .route("/upload/block", post(upload::block::handler))
        .route("/core/prune", post(prune_blocks::handler))
        .layer(cors_layer)
        .with_state(state)
}
