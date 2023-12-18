use std::error::Error;

use axum::body::HttpBody;
use axum::routing::post;
use axum::Router;
use tower_http::cors::CorsLayer;

mod blocks;
mod client_grant;
mod prune_blocks;
pub(crate) mod upload;

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
        .nest("/blocks", blocks::router(state.clone()))
        .route("/client_grant", post(client_grant::handler))
        .route("/upload", post(upload::handler))
        .route("/core/prune", post(prune_blocks::handler))
        .layer(cors_layer)
        .with_state(state)
}
