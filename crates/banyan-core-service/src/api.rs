use axum::Router;
use http::header::{ACCEPT, AUTHORIZATION, ORIGIN};
use http::Method;
use tower_http::cors::{Any, CorsLayer};

mod auth;
mod buckets;

pub fn router() -> Router {
    // todo: Ideally this would have a wrapper method to allow per route method configuration or
    // even better something that inspected the route matches and applied the correct method config
    // for that path...
    let cors_layer = CorsLayer::new()
        .allow_methods(vec![Method::GET, Method::DELETE, Method::POST, Method::PUT])
        .allow_headers(vec![AUTHORIZATION, ACCEPT, ORIGIN])
        // todo: add domain as a config option and make this configurable
        .allow_origin(Any)
        .allow_credentials(false);

    Router::new()
        .nest("/auth", auth::router())
        .nest("/buckets", buckets::router())
        .layer(cors_layer)
}
