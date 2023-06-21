use axum::routing::get;
use axum::Router;
use http::header::{ACCEPT, ORIGIN};
use http::Method;
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;

mod error;
mod handlers;
mod response;
mod service;

pub use error::HealthCheckError;
pub use response::Response as HealthCheckResponse;
pub use service::Service as HealthCheckService;

// requests to the healthcheck endpoints shouldn't contain anything other than headers, anything
// larger should be rejected.
const REQUEST_BODY_LIMIT: usize = 1_024;

pub fn router() -> Router {
    let cors_layer = CorsLayer::new()
        .allow_methods(vec![Method::GET])
        .allow_headers(vec![ACCEPT, ORIGIN])
        .allow_origin(Any)
        .allow_credentials(false);

    Router::new()
        .layer(RequestBodyLimitLayer::new(REQUEST_BODY_LIMIT))
        .layer(cors_layer)
        .route("/healthz", get(handlers::liveness_check))
        .route("/readyz", get(handlers::readiness_check))
}
