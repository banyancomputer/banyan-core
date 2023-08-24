use axum::Router;
use axum::routing::get;
use http::Method;
use http::header::{ACCEPT, ORIGIN};
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;

mod data_source;
mod liveness;
mod readiness;
mod version;

use crate::app::State;

/// Healthcheck endpoints generally shouldn't contain anything other than headers which are counted
/// among these bytes in the limit. Large requests here should always be rejected.
const HEALTHCHECK_REQUEST_SIZE_LIMIT: usize = 1_024;

pub fn router(state: State) -> Router<State> {
    let cors_layer = CorsLayer::new()
        .allow_methods(vec![Method::GET])
        .allow_headers(vec![ACCEPT, ORIGIN])
        .allow_origin(Any)
        .allow_credentials(false);

    Router::new()
        .layer(cors_layer)
        .layer(RequestBodyLimitLayer::new(HEALTHCHECK_REQUEST_SIZE_LIMIT))
        .route("/healthz", get(liveness::handler))
        .route("/readyz", get(readiness::handler))
        .route("/version", get(version::handler))
        .with_state(state)
}
