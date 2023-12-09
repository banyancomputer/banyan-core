use axum::body::HttpBody;
use axum::routing::get;
use axum::Router;
use http::header::{ACCEPT, ORIGIN};
use http::Method;
use serde::ser::StdError;
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;

mod error;
pub mod handlers;
mod responses;
mod service;

pub use error::Error as HealthCheckError;
pub use responses::Response as HealthCheckResponse;
pub use service::Service as HealthCheckService;

use crate::app::AppState;

// requests to the healthcheck endpoints shouldn't contain anything other than headers, anything
// larger should be rejected.
const REQUEST_BODY_LIMIT: usize = 1_024;

pub fn router<B>(state: AppState) -> Router<AppState, B>
where
    B: HttpBody + Send + 'static,
    Box<dyn StdError + Send + Sync + 'static>: From<B::Error>,
{
    let cors_layer = CorsLayer::new()
        .allow_methods(vec![Method::GET])
        .allow_headers(vec![ACCEPT, ORIGIN])
        .allow_origin(Any)
        .allow_credentials(false);

    Router::new()
        .route("/healthz", get(handlers::liveness_check))
        .route("/readyz", get(handlers::readiness_check))
        .route("/version", get(handlers::version))
        .layer(cors_layer)
        .layer(RequestBodyLimitLayer::new(REQUEST_BODY_LIMIT))
        .with_state(state)
}
