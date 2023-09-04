use axum::routing::{get, post};
use axum::Router;
use http::header::{ACCEPT, ORIGIN};
use http::{HeaderValue, Method};
use tower_http::cors::{Any, CorsLayer};

mod block_retrieval;
mod client_grant;
mod upload;

use crate::app::State;

pub fn router(state: State) -> Router<State> {
    let cors_layer = CorsLayer::new()
        .allow_methods(vec![Method::GET])
        .allow_headers(vec![ACCEPT, ORIGIN])
        .allow_origin(vec![state.hostname().as_str().parse().unwrap()])
        .allow_credentials(false);

    Router::new()
        .route("/blocks/:block_id", get(block_retrieval::handler))
        .route("/client_grant", post(client_grant::handler))
        .route("/upload", post(upload::handler))
        .layer(cors_layer)
        .with_state(state)
}
