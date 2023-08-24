use axum::Router;
//use axum::routing::get;
use http::Method;
use http::header::{ACCEPT, AUTHORIZATION, ORIGIN};
use tower_http::cors::{Any, CorsLayer};

use crate::app::State;

pub fn router(state: State) -> Router<State> {
    let cors_layer = CorsLayer::new()
        .allow_methods(vec![Method::GET, Method::POST])
        .allow_headers(vec![ACCEPT, AUTHORIZATION, ORIGIN])
        .allow_origin(Any)
        .allow_credentials(true);

    Router::new()
        .layer(cors_layer)
        .with_state(state)
}
