use std::sync::Arc;
use std::time::Duration;

use axum::error_handling::HandleErrorLayer;
use axum::extract::DefaultBodyLimit;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, get_service};
use axum::{Json, Router};
use axum::{Server, ServiceExt};
use futures::future::join_all;
use http::header;
use tokio::fs::File;
use tokio::io::AsyncReadExt;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tower::ServiceBuilder;
use tower_http::request_id::MakeRequestUuid;
use tower_http::sensitive_headers::{
    SetSensitiveRequestHeadersLayer, SetSensitiveResponseHeadersLayer,
};
use tower_http::trace::{DefaultMakeSpan, DefaultOnFailure, DefaultOnResponse, TraceLayer};
use tower_http::validate_request::ValidateRequestHeaderLayer;
use tower_http::{LatencyUnit, ServiceBuilderExt};
use tracing::Level;

use axum::body::{boxed, BoxBody};
use axum::http::Request;
use axum::response::Response;
use tower::ServiceExt as OtherServiceExt;

use crate::app::{AppState, Config};
use crate::tasks::start_background_workers;
use crate::{api, auth, health_check, hooks};
use tower_http::services::ServeDir;
use tower_http::services::ServeFile;

// TODO: might want a longer timeout in some parts of the API and I'd like to be able customize a
// few layers eventually such as CORS and request timeouts but that's for something down the line
const REQUEST_TIMEOUT_SECS: u64 = 90;

// The timestamp of the current TOS file.
// Used to select the correct TOS file to serve.
// Forrmatted: YYYYMMDD
const CURRENT_TOS_DATE: &str = "20231127";

// TODO: probably want better fallback error pages...
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

    // TODO: I want to log the error chain, but there is some weird trait shenangigans that need to
    // be worked through to call the collect_error_messages function
    tracing::error!(error_msg = %error, "unhandled error");

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal error".to_owned(),
    )
}

pub async fn graceful_shutdown_blocker() -> (JoinHandle<()>, watch::Receiver<()>) {
    use tokio::signal::unix;

    let mut sig_int_handler =
        unix::signal(unix::SignalKind::interrupt()).expect("to be able to install signal handler");
    let mut sig_term_handler =
        unix::signal(unix::SignalKind::terminate()).expect("to be able to install signal handler");

    let (tx, rx) = tokio::sync::watch::channel(());
    let handle = tokio::spawn(async move {
        // TODO: need to follow k8s signal handling rules for these different signals
        tokio::select! {
            _ = sig_int_handler.recv() => tracing::debug!("gracefully exiting on an interrupt signal"),
            _ = sig_term_handler.recv() => tracing::debug!("gracefully exiting on an terminate signal"),
        }

        let _ = tx.send(());
    });

    (handle, rx)
}

async fn login_page_handler<B: std::marker::Send + 'static>(
    req: Request<B>,
) -> Result<Response, (StatusCode, String)> {
    match ServeFile::new("./dist/login.html").oneshot(req).await {
        Ok(res) => Ok(res.map(boxed)),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong serving the login page: {}", err),
        )),
    }
}

async fn tos_handler<B: std::marker::Send + 'static>(
    _req: Request<B>,
) -> Result<Response<BoxBody>, (StatusCode, String)> {
    // Read the TOS content from the file specified by CURRENT_TOS_DATE
    let current_tos_date = CURRENT_TOS_DATE;
    let file_path = format!("./dist/tos/{}_tos.txt", current_tos_date);
    let mut file = match File::open(&file_path).await {
        Ok(file) => file,
        Err(err) => {
            tracing::error!("Failed to open TOS file: {err}");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "A backend service issue occurred".to_string(),
            ));
        }
    };

    let mut tos_content = String::new();
    if let Err(err) = file.read_to_string(&mut tos_content).await {
        tracing::error!("Failed to read TOS file: {err}");
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "A backend service issue occurred".to_string(),
        ));
    }

    let tos_response = serde_json::json!({
        // Text context
        "tos_content": tos_content,
        // YYYYMMDD formatted date
        "current_tos_date": current_tos_date,
    });

    Ok((StatusCode::OK, Json(tos_response)).into_response())
}

pub async fn run(config: Config) {
    let (shutdown_handle, mut shutdown_rx) = graceful_shutdown_blocker().await;
    let listen_addr = config.listen_addr();
    let app_state = AppState::from_config(&config)
        .await
        .expect("app state to be created");

    let worker_handle = start_background_workers(app_state.clone(), shutdown_rx.clone())
        .await
        .expect("background workers to start");

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
                .include_headers(false)
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
        .layer(DefaultBodyLimit::disable())
        .layer(ValidateRequestHeaderLayer::accept("application/json"))
        .layer(SetSensitiveResponseHeadersLayer::from_shared(
            sensitive_headers,
        ));

    let static_assets = ServeDir::new("dist").not_found_service(
        get_service(ServeFile::new("./dist/index.html")).handle_error(|_| async move {
            (StatusCode::INTERNAL_SERVER_ERROR, "internal server error")
        }),
    );

    let root_router = Router::new()
        .nest("/api/v1", api::router(app_state.clone()))
        .nest("/auth", auth::router(app_state.clone()))
        .nest("/hooks", hooks::router(app_state.clone()))
        .nest("/_status", health_check::router(app_state.clone()))
        .route("/login", get(login_page_handler))
        .route("/tos", get(tos_handler))
        .with_state(app_state)
        .fallback_service(static_assets);

    let app = middleware_stack.service(root_router);

    tracing::info!(listen_addr = ?listen_addr, "service starting up");

    let web_handle: JoinHandle<()> = tokio::spawn(async move {
        Server::bind(&listen_addr)
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
