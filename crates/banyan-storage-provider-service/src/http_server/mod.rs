use std::time::Duration;

use axum::error_handling::HandleErrorLayer;
use axum::extract::DefaultBodyLimit;
use axum::handler::HandlerWithoutStateExt;
use axum::{Router, Server, ServiceExt};
use futures::future::join_all;
use http::header;
use tokio::task::JoinHandle;
use tower::ServiceBuilder;
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower_http::request_id::MakeRequestUuid;
use tower_http::sensitive_headers::{
    SetSensitiveRequestHeadersLayer, SetSensitiveResponseHeadersLayer,
};
use tower_http::services::ServeDir;
use tower_http::trace::{DefaultMakeSpan, DefaultOnFailure, DefaultOnResponse, TraceLayer};
use tower_http::validate_request::ValidateRequestHeaderLayer;
use tower_http::{LatencyUnit, ServiceBuilderExt};
use tracing::Level;

use crate::app::{AppState, Config};
use crate::tasks::start_background_workers;
use crate::{api, health_check};

mod error_handlers;
mod shutdown_blocker;

const REQUEST_GRACE_PERIOD: Duration = Duration::from_secs(10);

/// The largest size content that any client can send us before we reject it. This is a pretty
/// heavily restricted default but most JSON responses are relatively tiny.
const REQUEST_MAX_SIZE: usize = 256 * 1_024;

/// The maximum number of seconds that any individual request can take before it is dropped with an
/// error.
const REQUEST_TIMEOUT: Duration = std::time::Duration::from_secs(15);

const SENSITIVE_HEADERS: &[http::HeaderName] = &[
    header::AUTHORIZATION,
    header::COOKIE,
    header::PROXY_AUTHORIZATION,
    header::SET_COOKIE,
];

fn create_trace_layer(log_level: Level) -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>> {
    TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(log_level))
        .on_response(
            DefaultOnResponse::new()
                .include_headers(false)
                .level(log_level)
                .latency_unit(LatencyUnit::Micros),
        )
        .on_failure(DefaultOnFailure::new().latency_unit(LatencyUnit::Micros))
}

/// Run the App over HTTP
pub async fn run(config: Config) {
    // Initialize a blocker that will allow us to gracefully shutdown
    let (shutdown_handle, mut shutdown_rx) = shutdown_blocker::graceful_shutdown_blocker().await;
    // Specify log level for tracing
    let trace_layer = create_trace_layer(config.log_level());
    // Create our middleware stack
    // The order of these layers and configuration extensions was carefully chosen as they will see
    // the requests to responses effectively in the order they're defined.
    let middleware_stack = ServiceBuilder::new()
        // Tracing and log handling get setup before anything else
        .layer(trace_layer)
        .layer(HandleErrorLayer::new(error_handlers::server_error_handler))
        // Ensure any sensitive headers are stripped before we do any logging
        .layer(SetSensitiveRequestHeadersLayer::from_shared(
            SENSITIVE_HEADERS.into(),
        ))
        // Set a timeout for any request that comes in -- this is a hard timeout and will drop the request
        .timeout(REQUEST_TIMEOUT)
        // Pre-empt overloading by responding with a 503 resources are unavailable
        .load_shed()
        // Restrict the number of concurrent in flight requests -- should reflect the number of concurrent requests your service can handle.
        .concurrency_limit(1024)
        // Make sure our request has a unique identifier if we don't already have one.
        // Propgate that identifier to any downstream services to avoid untrusted injection of this header.
        .set_x_request_id(MakeRequestUuid)
        .propagate_x_request_id()
        // Default request size. Individual handlers can opt-out of this limit, see api/upload.rs for an example.
        .layer(DefaultBodyLimit::max(REQUEST_MAX_SIZE))
        // TODO: is this desired?
        // Restrict requests to only those that are JSON
        .layer(ValidateRequestHeaderLayer::accept("application/json"))
        // Filter out any sensitive headers from the response
        .layer(SetSensitiveResponseHeadersLayer::from_shared(
            SENSITIVE_HEADERS.into(),
        ));
    // Create a new instance of our application state
    let app_state = AppState::from_config(&config)
        .await
        .expect("app state to be created");
    // TODO: service index.html from dist if not found
    // Start background workers
    let worker_handle = start_background_workers(app_state.clone(), shutdown_rx.clone())
        .await
        .expect("background workers to start");
    // Serve static assets
    let static_assets =
        ServeDir::new("dist").not_found_service(error_handlers::not_found_handler.into_service());
    // Create our root router for handling requests
    let root_router = Router::new()
        .nest("/api/v1", api::router(app_state.clone()))
        .nest("/_status", health_check::router(app_state.clone()))
        .with_state(app_state)
        .fallback_service(static_assets);
    // Create our app service
    let app = middleware_stack.service(root_router);

    tracing::info!(addr = ?config.listen_addr(), "service listening");

    let web_handle: JoinHandle<()> = tokio::spawn(async move {
        Server::bind(&config.listen_addr())
            .serve(app.into_make_service())
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.changed().await;
            })
            .await
            .expect("server to exit cleanly upon completion");
    });

    // wait for a shutdown signal, let everything run in the background
    let _ = shutdown_handle.await;

    let _ = tokio::time::timeout(
        Duration::from_secs(5),
        join_all([worker_handle, web_handle]),
    )
    .await;
}
