use std::time::Duration;

use axum::error_handling::HandleErrorLayer;
use axum::extract::DefaultBodyLimit;
use axum::Router;
use axum::{Server, ServiceExt};
use http::header;
use tokio::signal::unix::{signal, SignalKind};
use tower::ServiceBuilder;
use tower_http::classify::{ServerErrorsAsFailures, SharedClassifier};
use tower_http::request_id::MakeRequestUuid;
use tower_http::sensitive_headers::{
    SetSensitiveRequestHeadersLayer, SetSensitiveResponseHeadersLayer,
};
use tower_http::trace::{DefaultMakeSpan, DefaultOnFailure, DefaultOnResponse, TraceLayer};
use tower_http::validate_request::ValidateRequestHeaderLayer;
use tower_http::{LatencyUnit, ServiceBuilderExt};
use tracing::Level;

use crate::app::{Config, Error, State};
use crate::{api, health_check};

mod error_handlers;

const REQUEST_GRACE_PERIOD: Duration = Duration::from_secs(10);

/// The largest size content that any client can send us before we reject it. This is a pretty
/// heavily restricted default but most JSON responses are relatively tiny.
const REQUEST_MAX_SIZE: usize = 256 * 1_024;

/// The maximum number of seconds that any individual request can take before it is dropped with an
/// error.
const REQUEST_TIMEOUT_SECS: u64 = 15;

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
                .latency_unit(LatencyUnit::Nanos),
        )
        .on_failure(DefaultOnFailure::new().latency_unit(LatencyUnit::Micros))
}

/// Follow k8s signal handling rules for these different signals. The order of shutdown events are:
///
/// 1. Pod is set to the "Terminating" state and removed from the endpoints list of all services,
///    new traffic should stop appearing
/// 2. The preStop Hook is executed if configured, can send a command or an http request. Should be
///    implemented if SIGTERM doesn't gracefully shutdown your app. Simultaneously k8s will start
///    issuing endpoint update commands indicating the service should be removed from load
///    balancers.
/// 3. SIGTERM signal is sent to the pod, your service should start shutting down cleanly, service
///    has 30 seconds to perform any clean up, shutdown, and state saving. The service may still
///    receive requests for up to 10 seconds on GKE according to some blog post. This would make
///    sense as the event time needs to propagate through the system and is supported by this quote
///    about service meshes:
///
///    > Since the components might be busy doing something else, there is no guarantee on how
///    > long it will take to remove the IP address from their internal state.
///
///    I've seen recommendations that the readiness probe should start failing here and others
///    reporting that won't do anything. As far as I can tell failing the readiness probe here
///    makes sense and at worse will do nothing.
///
///    It seems that the common recommendation here is to wait for 10-15 seconds in the
///    graceperiod, with readiness failing, then exit
/// 4. If the container doesn't exit on its own after 30 seconds it will receive a SIGKILL which we
///    can't respond to, we just get killed.
///
/// This also handles SIGINT which K8s doesn't issue, those will be coming from users running the
/// server locally and should shut the server down immediately.
async fn graceful_shutdown_blocker() {
    let mut sigint = signal(SignalKind::interrupt()).unwrap();
    let mut sigterm = signal(SignalKind::terminate()).unwrap();

    tokio::select! {
        _ = sigint.recv() => {
            tracing::debug!("gracefully exiting immediately on SIGINT");
            return;
        }
        _ = sigterm.recv() => tracing::debug!("initiaing graceful shutdown with delay on SIGTERM"),
    }

    // todo: fail the readiness checks

    tokio::time::sleep(REQUEST_GRACE_PERIOD).await
}

pub async fn run(config: Config) -> Result<(), Error> {
    let trace_layer = create_trace_layer(config.log_level());

    // The order of these layers and configuration extensions was carefully chosen as they will see
    // the requests to responses effectively in the order they're defined.
    let middleware_stack = ServiceBuilder::new()
        // Tracing and log handling get setup before anything else
        .layer(trace_layer)
        .layer(HandleErrorLayer::new(error_handlers::server_error_handler))
        // From here on out our requests might be logged, ensure any sensitive headers are stripped
        // before we do any logging
        .layer(SetSensitiveRequestHeadersLayer::from_shared(
            SENSITIVE_HEADERS.into(),
        ))
        // If requests are queued or take longer than this duration we want the cut them off
        // regardless of any other protections that are inplace
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        // If any future services or middleware indicate they're not available, reject them with a
        // service too busy error
        .load_shed()
        // Restrict the number of concurrent in flight requests, desired value for this is going to
        // vary from service to service, make sure it reflects the number of concurrent requests
        // your service can handle.
        .concurrency_limit(1024)
        // Make sure our request has a unique identifier if we don't already have one. This does
        // allow our upstream to arbitrarily set headers so this service should have protection
        // against arbitrary untrusted injections of this header.
        .set_x_request_id(MakeRequestUuid)
        .propagate_x_request_id()
        // By default limit any request to this size. Individual handlers can opt-out of this limit
        // if they so choose (such as an upload handler).
        .layer(DefaultBodyLimit::max(REQUEST_MAX_SIZE))
        // Our clients should only ever be sending us JSON requests, any other type is an error.
        // This won't be true of all APIs and this will accept the wildcards sent by most clients.
        // Debatable whether I actually want this...
        .layer(ValidateRequestHeaderLayer::accept("application/json"))
        // Finally make sure any responses successfully generated from our service is also
        // filtering out any sensitive headers from our logs.
        .layer(SetSensitiveResponseHeadersLayer::from_shared(
            SENSITIVE_HEADERS.into(),
        ));

    let state = State::from_config(&config).await?;
    let root_router = Router::new()
        .nest("/api", api::router(state.clone()))
        .nest("/_status", health_check::router(state.clone()))
        .with_state(state)
        .fallback(error_handlers::not_found_handler);
    let app = middleware_stack.service(root_router);

    tracing::info!(addr = ?config.listen_addr(), "server listening");
    Server::bind(&config.listen_addr())
        .serve(app.into_make_service())
        .with_graceful_shutdown(graceful_shutdown_blocker())
        .await?;

    Ok(())
}
