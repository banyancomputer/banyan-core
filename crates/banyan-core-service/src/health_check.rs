use axum::routing::get;
use axum::Router;
use http::header::{ACCEPT, ORIGIN};
use http::Method;
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;

mod error;
mod handlers;
mod responses;
mod service;

pub use error::Error as HealthCheckError;
pub use responses::Response as HealthCheckResponse;
pub use responses::VersionResponse;
pub use service::Service as HealthCheckService;

use crate::app_state::AppState;

// requests to the healthcheck endpoints shouldn't contain anything other than headers, anything
// larger should be rejected.
const REQUEST_BODY_LIMIT: usize = 1_024;

pub fn router(state: AppState) -> Router<AppState> {
    let cors_layer = CorsLayer::new()
        .allow_methods(vec![Method::GET])
        .allow_headers(vec![ACCEPT, ORIGIN])
        .allow_origin(Any)
        .allow_credentials(false);

    Router::new()
        .layer(cors_layer)
        .layer(RequestBodyLimitLayer::new(REQUEST_BODY_LIMIT))
        .route("/healthz", get(handlers::liveness_check))
        .route("/readyz", get(handlers::readiness_check))
        .route("/version", get(handlers::version))
        .route("/work_test", get(handlers::work_test))
        .with_state(state)
}
