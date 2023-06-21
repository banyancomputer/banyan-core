use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::error_handling::HandleErrorLayer;
use axum::response::IntoResponse;
use axum::{Router, Server, ServiceExt};
use http::{header, StatusCode};
use tower::ServiceBuilder;
use tower_http::request_id::MakeRequestUuid;
use tower_http::sensitive_headers::{
    SetSensitiveRequestHeadersLayer, SetSensitiveResponseHeadersLayer,
};
use tower_http::trace::{DefaultMakeSpan, DefaultOnFailure, DefaultOnResponse, TraceLayer};
use tower_http::{LatencyUnit, ServiceBuilderExt};
use tracing::Level;

mod api;
mod health_check;
mod static_files;

// todo: might want a longer timeout in some parts of the API and I'd like to be able customize a
// few layers eventually such as CORS and request timeouts but that's for something down the line
const REQUEST_TIMEOUT_SECS: u64 = 30;

// todo: probably want better fallback error pages...
async fn handle_error(error: tower::BoxError) -> impl IntoResponse {
    if error.is::<tower::timeout::error::Elapsed>() {
        return (StatusCode::REQUEST_TIMEOUT, "request timeout".to_owned());
    }

    if error.is::<tower::load_shed::error::Overloaded>() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "service is overloaded".to_owned(),
        );
    }

    // todo: I want to log the error chain, but there is some weird trait shenangigans that need to
    // be worked through to call the collect_error_messages function
    tracing::error!(error_msg = %error, "unhandled error");

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal error".to_owned(),
    )
}

async fn graceful_shutdown_blocker() {
    use tokio::signal::unix;

    let mut sig_int_handler =
        unix::signal(unix::SignalKind::interrupt()).expect("to be able to install signal handler");
    let mut sig_term_handler =
        unix::signal(unix::SignalKind::terminate()).expect("to be able to install signal handler");

    // todo: need to follow k8s signal handling rules for these different signals
    tokio::select! {
        _ = sig_int_handler.recv() => tracing::debug!("gracefully exiting on an interrupt signal"),
        _ = sig_term_handler.recv() => tracing::debug!("gracefully exiting on an terminate signal"),
    }
}

pub async fn serve() -> anyhow::Result<()> {
    let sensitive_headers: Arc<[_]> = Arc::new([
        header::AUTHORIZATION,
        header::COOKIE,
        header::PROXY_AUTHORIZATION,
        header::SET_COOKIE,
    ]);

    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_response(
            DefaultOnResponse::new()
                .include_headers(true)
                .level(Level::INFO)
                .latency_unit(LatencyUnit::Micros),
        )
        .on_failure(DefaultOnFailure::new().latency_unit(LatencyUnit::Micros));

    let middleware_stack = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(handle_error))
        .load_shed()
        .concurrency_limit(1024)
        .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
        .layer(SetSensitiveRequestHeadersLayer::from_shared(Arc::clone(
            &sensitive_headers,
        )))
        .set_x_request_id(MakeRequestUuid)
        .layer(trace_layer)
        .propagate_x_request_id()
        .layer(SetSensitiveResponseHeadersLayer::from_shared(
            sensitive_headers,
        ));

    let root_router = Router::new()
        .fallback_service(static_files::router())
        .nest("/api/v1", api::router())
        .nest("/_status", health_check::router());

    let addr: SocketAddr = "[::]:3000".parse()?;
    let app = middleware_stack.service(root_router);

    tracing::info!(addr = ?addr, "server listening");

    Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(graceful_shutdown_blocker())
        .await?;

    Ok(())
}
