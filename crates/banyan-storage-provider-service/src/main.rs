use std::time::Duration;

use axum::{Json, Router, Server};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use futures::future::join_all;
use sqlx::SqlitePool;
use tokio::task::JoinHandle;
use tokio::sync::watch;
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

#[tokio::main]
async fn main() {
    let (non_blocking_writer, _guard) = tracing_appender::non_blocking(std::io::stderr());
    let env_filter = EnvFilter::builder()
        .with_default_directive(Level::INFO.into())
        .from_env_lossy();

    let stderr_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_writer(non_blocking_writer)
        .with_filter(env_filter);

    tracing_subscriber::registry().with(stderr_layer).init();

    // No authentication required to the API or status endpoint
    //
    // /_status/healthz
    //   * { "health_status": "green/yellow/red", "database": "healthy", "job_queue": 0 }
    // /api/v1/alerts
    //   * List of currently active alerts
    //   * { "alerts": [{"id": "<uuid>", "msg": "Something has gone wrong!", "type": "setup_required", "triggered_at": "<date time>"}] }
    // /api/v1/alerts/history
    //   * { "alerts": [{"id": "<uuid>", "msg": "Something has gone wrong!", "type": "deal_expired", "triggered_at": "<date time>", "resolved_at": "<date time>"}] }
    // /api/v1/config
    //   * { "friendly_name": "Vault42", "platform": {"name": "vault_42", "id": "<uuid>"},
    //   "settings": { "billing_start_day": 5, "time_zone": "America/New_York" } }
    // /api/v1/deals/available
    //   * size in bytes, price in USD * 10_000 (example is $2.45)
    //   * [{"id": "<uuid>", "size": 1234567, "payment": 245000, "accept_by": "<date time>", "seal_by": "<date time>"}]
    // /api/v1/deals/:deal_id
    //   * size in bytes
    //   * price in USD * 10_000
    //   * status in ('available', 'pending', 'sealed', 'violated', 'expired', 'cancelled', 'completed'),
    //     available will never be returned by this API endpoint only by the 'available' endpoint
    //   * "seal_by" only present in status "pending"
    //   * "sealed_at" is present for state 'sealed', 'violated', 'expired'
    //   * "expires_at" is present for state 'sealed, 'violated', 'expired'
    //   * "cancelled_at" is present for state 'cancelled'
    //   * [{"id": "<uuid>", "status": "sealed", "size": 1234567, "payment": 245000, "accepted_at": "<date time>", "seal_by": "<date time>", "expires_at": "<date time>", "cancelled_at": "<date time>"}]
    // /api/v1/deals/:deal_id/cancel
    //   * Should only work for deals in waiting for seal, no body, 204 on success, no content
    // /api/v1/deals/available/:deal_id/accept
    //   * 204 on success, no content
    // /api/v1/deals/available/:deal_id/ignore
    //   * 204 on success, no content
    // /api/v1/metrics/current
    //   * used/available storage amounts, bandwidth ingress/egress all in bytes
    //   * {"storage": {"used": 123456, "available": 123456}, "bandwidth": {"ingress": 67070, "egress": 80123}, "deals": {"accepted": 67, "sealed_amt": 12355}}
    // /api/v1/metrics/bandwidth/daily
    //   * [{"timestamp": "<date time>", "ingress": 12345, "egress": 89023}, ... ]
    // /api/v1/metrics/storage/daily
    //   * [{"timestamp": "<date time>", "consumed": 12345, "available": 89023}, ... ]

    let (shutdown_handle, mut shutdown_rx) = graceful_shutdown_blocker().await;

    let state = State::new().await;

    let app = Router::new()
        .with_state(state)
        .fallback(not_found_handler);

    let listen_addr = "127.0.0.1:3003".parse().expect("valid");
    let web_handle: JoinHandle<()> = tokio::spawn(async move {
        Server::bind(&listen_addr)
            .serve(app.into_make_service())
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.changed().await;
            })
            .await
            .expect("server to exit cleanly upon completion");
    });

    let _ = shutdown_handle.await;
    let _ = tokio::time::timeout(Duration::from_secs(5), join_all([web_handle])).await;
}

#[derive(Clone)]
pub struct State {
    database: SqlitePool,
}

impl State {
    pub async fn new() -> Self {
        Self {
            database: SqlitePool::connect("sqlite::memory:").await.expect("valid"),
        }
    }

    pub fn database(&self) -> SqlitePool {
        self.database.clone()
    }
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

async fn not_found_handler() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({"status": "not found"})),
    )
}
