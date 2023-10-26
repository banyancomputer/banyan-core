use std::time::Duration;

use axum::extract::Path;
use axum::handler::HandlerWithoutStateExt;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router, Server};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use time::OffsetDateTime;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tower_http::services::ServeDir;
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};
use uuid::Uuid;

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
    // /api/v1/deals/available/:deal_id/download
    //   * starts file download if present...
    // /api/v1/deals/available/:deal_id/ignore
    //   * 204 on success, no content
    // /api/v1/deals/:deal_id/proof
    //   * sector ID is a u64
    //   * future work will include the actual PoSt and PoRep proofs
    //   * {"id": "<uuid>", "sector_id": 1234616, "sealed_cid": "<sector cid>", "merkle_root": "<digest>", "timestamp": "<date time>"}
    // /api/v1/metrics/current
    //   * used/available storage amounts, bandwidth ingress/egress all in bytes
    //   * {"storage": {"used": 123456, "available": 123456}, "bandwidth": {"ingress": 67070, "egress": 80123}, "deals": {"accepted": 67, "sealed_amt": 12355}}
    // /api/v1/metrics/bandwidth/daily
    //   * [{"timestamp": "<date time>", "ingress": 12345, "egress": 89023}, ... ]
    // /api/v1/metrics/storage/daily
    //   * [{"timestamp": "<date time>", "consumed": 12345, "available": 89023}, ... ]

    let (shutdown_handle, mut shutdown_rx) = graceful_shutdown_blocker().await;

    let state = AppState::new().await;

    let static_assets = ServeDir::new("dist").not_found_service(not_found_handler.into_service());

    let app = Router::new()
        .route("/_status/healthz", get(healthcheck_handler))
        .route("/api/v1/alerts", get(alerts_handler))
        .route("/api/v1/alerts/history", get(alert_history_handler))
        .route("/api/v1/deals/available", get(deal_available_handler))
        .route("/api/v1/deals/:deal_id", get(deal_single_handler))
        .route("/api/v1/deals/:deal_id/accept", get(deal_accept_handler))
        .route("/api/v1/deals/:deal_id/cancel", get(deal_cancel_handler))
        .route(
            "/api/v1/deals/:deal_id/download",
            get(deal_download_handler),
        )
        .route("/api/v1/deals/:deal_id/ignore", get(deal_ignore_handler))
        .route("/api/v1/deals/:deal_id/proof", get(deal_proof_handler))
        .route("/api/v1/metrics/current", get(metrics_current_handler))
        .route(
            "/api/v1/metrics/bandwidth/daily",
            get(metrics_bandwidth_daily_handler),
        )
        .route(
            "/api/v1/metrics/storage/daily",
            get(metrics_storage_daily_handler),
        )
        .with_state(state)
        .fallback_service(static_assets);

    let listen_addr = "127.0.0.1:3003".parse().expect("valid");
    tracing::info!("service listening on {listen_addr}");
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
pub struct AppState {
    database: SqlitePool,
}

impl AppState {
    pub async fn new() -> Self {
        Self {
            database: SqlitePool::connect("sqlite::memory:").await.expect("valid"),
        }
    }

    pub fn database(&self) -> SqlitePool {
        self.database.clone()
    }
}

pub async fn alerts_handler() -> Response {
    let resp_msg = serde_json::json!([Alert::random(), Alert::random(), Alert::random()]);
    (StatusCode::OK, Json(resp_msg)).into_response()
}

pub async fn alert_history_handler() -> Response {
    let resp_msg = serde_json::json!([
        Alert::random(),
        Alert::random_resolved(),
        Alert::random_resolved(),
    ]);
    (StatusCode::OK, Json(resp_msg)).into_response()
}

pub async fn config_handler() -> Response {
    let resp_msg = serde_json::json!({
        "friendly_name": "Vault42",
        "platform": {
            "id": Uuid::new_v4(),
            "name": "vault_42",
        },
        "settings": {
            "current_billing_start_day": 5, // valid 1-28,
            "next_billing_start_day": 9, // valid 1-28,
            "time_zone": "America/New_York",
        },
    });

    (StatusCode::OK, Json(resp_msg)).into_response()
}

pub async fn deal_accept_handler(Path(deal_id): Path<Uuid>) -> Response {
    let resp_msg = serde_json::json!({"id": deal_id, "msg": "in progress"});
    (StatusCode::OK, Json(resp_msg)).into_response()
}

pub async fn deal_available_handler() -> Response {
    let resp_msg = serde_json::json!({"msg": "in progress"});
    (StatusCode::OK, Json(resp_msg)).into_response()
}

pub async fn deal_cancel_handler(Path(deal_id): Path<Uuid>) -> Response {
    let resp_msg = serde_json::json!({"id": deal_id, "msg": "in progress"});
    (StatusCode::OK, Json(resp_msg)).into_response()
}

pub async fn deal_download_handler(Path(_deal_id): Path<Uuid>) -> Response {
    todo!()
}

pub async fn deal_ignore_handler(Path(deal_id): Path<Uuid>) -> Response {
    let resp_msg = serde_json::json!({"id": deal_id, "msg": "in progress"});
    (StatusCode::OK, Json(resp_msg)).into_response()
}

pub async fn deal_proof_handler(Path(deal_id): Path<Uuid>) -> Response {
    let resp_msg = serde_json::json!({"id": deal_id, "msg": "in progress"});
    (StatusCode::OK, Json(resp_msg)).into_response()
}

pub async fn deal_single_handler(Path(deal_id): Path<Uuid>) -> Response {
    let resp_msg = serde_json::json!({"id": deal_id, "msg": "in progress"});
    (StatusCode::OK, Json(resp_msg)).into_response()
}

pub async fn healthcheck_handler() -> Response {
    let resp_msg = serde_json::json!({
        "health_status": HealthCheckStatus::Green,
        "database": "healthy",
        "job_queue": 5,
    });

    (StatusCode::OK, Json(resp_msg)).into_response()
}

pub async fn metrics_current_handler() -> Response {
    let resp_msg = serde_json::json!({"msg": "in progress"});
    (StatusCode::OK, Json(resp_msg)).into_response()
}

pub async fn metrics_bandwidth_daily_handler() -> Response {
    let resp_msg = serde_json::json!({"msg": "in progress"});
    (StatusCode::OK, Json(resp_msg)).into_response()
}

pub async fn metrics_storage_daily_handler() -> Response {
    let resp_msg = serde_json::json!({"msg": "in progress"});
    (StatusCode::OK, Json(resp_msg)).into_response()
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

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum HealthCheckStatus {
    Red,
    Yellow,
    Green,
}

#[derive(Deserialize, Serialize)]
struct Alert {
    id: Uuid,

    #[serde(rename = "msg")]
    message: String,

    severity: AlertSeverity,
    r#type: AlertType,

    triggered_at: OffsetDateTime,
    resolved_at: Option<OffsetDateTime>,
}

impl Alert {
    fn random() -> Self {
        Self {
            id: Uuid::new_v4(),
            severity: AlertSeverity::Warning,
            message: "here's some alert text. Something is going on".to_string(),
            r#type: AlertType::SetupRequired,
            triggered_at: OffsetDateTime::now_utc(),
            resolved_at: None,
        }
    }

    fn random_resolved() -> Self {
        Self {
            id: Uuid::new_v4(),
            severity: AlertSeverity::Warning,
            message: "here's some alert text. Something is going on".to_string(),
            r#type: AlertType::SetupRequired,
            triggered_at: OffsetDateTime::now_utc(),
            resolved_at: Some(OffsetDateTime::now_utc()),
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum AlertSeverity {
    Warning,
    Error,
    Fatal,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum AlertType {
    DealExpired,
    SetupRequired,
}

async fn not_found_handler() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({"status": "not found"})),
    )
}
