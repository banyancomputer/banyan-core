use axum::Router;
use axum::routing::post;
use http::Method;
use http::header::{ACCEPT, AUTHORIZATION, ORIGIN};
use tower_http::cors::{Any, CorsLayer};

mod client_grant;
mod upload;

use crate::app::State;

pub fn router(state: State) -> Router<State> {
    let cors_layer = CorsLayer::new()
        .allow_methods(vec![Method::GET, Method::POST])
        .allow_headers(vec![ACCEPT, AUTHORIZATION, ORIGIN])
        .allow_origin(Any)
        .allow_credentials(true);

    Router::new()
        .layer(cors_layer)
        .route("/client_grant", post(client_grant::handler))
        .route("/upload", post(upload::handler))
        .with_state(state)
}
