use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::time::Duration;

use axum::body::HttpBody;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use http::{HeaderMap, HeaderValue};
use rand::Rng;
use serde::Serialize;
use time::ext::NumericalDuration;
use time::{Date, OffsetDateTime};
use tower_http::cors::CorsLayer;
use uuid::Uuid;

mod auth;
mod block_present;
mod block_retrieval;
mod client_grant;
mod deals;
mod hooks;
mod models;
mod prune_blocks;
mod upload;

pub use deals::DealQuery;

use crate::app::AppState;

const CURRENCY_MULTIPLIER: usize = 10_000;

const PRICE_PER_TIB: usize = 2 * CURRENCY_MULTIPLIER;

pub fn router<B>(state: AppState) -> Router<AppState, B>
where
    B: HttpBody + Send + 'static,
    B::Data: Send,
    bytes::Bytes: From<B::Data>,
    Box<dyn Error + Send + Sync + 'static>: From<B::Error>,
{
    let cors_layer = CorsLayer::very_permissive();

    Router::new()
        .nest("/auth", auth::router(state.clone()))
        // TODO: Should we place these behind a new prefix?
        // Client Storage API routes
        .route("/blocks/:block_id", get(block_retrieval::handler))
        .route("/blocks/present", post(block_present::handler))
        .route("/client_grant", post(client_grant::handler))
        .route("/upload", post(upload::handler))
        .nest("/hooks", hooks::router(state.clone()))
        .nest("/deals", deals::router(state.clone()))
        .route("/upload/new", post(upload::new::handler))
        .route("/upload/block", post(upload::block::handler))
        .route("/core/prune", post(prune_blocks::handler))
        // Storage provider API routes
        .route("/alerts", get(alerts_handler))
        .route("/alerts/history", get(alert_history_handler))
        .route("/metrics/current", get(metrics_current_handler))
        .route(
            "/metrics/bandwidth/daily",
            get(metrics_bandwidth_daily_handler),
        )
        .route("/metrics/storage/daily", get(metrics_storage_daily_handler))
        .layer(cors_layer)
        .with_state(state)
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

pub async fn deal_download_handler(Path(deal_id): Path<Uuid>) -> Response {
    let mut rng = rand::thread_rng();

    let mut headers = HeaderMap::new();

    let disposition =
        HeaderValue::from_str(format!("attachment; filename=\"{deal_id}.car\"").as_str()).unwrap();
    headers.insert(CONTENT_DISPOSITION, disposition);
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/vnd.ipld.car; version=2"),
    );

    let mut data_to_seal = [0u8; 1024];
    rng.fill(&mut data_to_seal);

    (StatusCode::OK, headers, data_to_seal).into_response()
}

pub async fn deal_ignore_handler(Path(_deal_id): Path<Uuid>) -> Response {
    (StatusCode::NO_CONTENT, ()).into_response()
}

pub async fn deal_proof_handler(Path(deal_id): Path<Uuid>) -> Response {
    // todo: include PoSt and PoRep details
    let resp_msg = serde_json::json!({
        "id": deal_id,
        "sector_id": 123456789u64,
        "sealed_cid": "uAVUSIPETJqF9uI82g0Gk1Dk_eAJ0NxXGvFJ1Gpx2W1E0MDyV",
        "merkle_root": "22595ccbf9d38fe952ddefae13a2be1584c8afcf971a17e9a6e1ee902cb79ed41430e27a14f8d6ffb469c9cb53baec89aa29ba4e0fc4b14d8cdbac73f1a0080b",
        "timestamp": OffsetDateTime::now_utc(),
    });

    (StatusCode::OK, Json(resp_msg)).into_response()
}

pub async fn deal_single_handler(Path(deal_id): Path<Uuid>) -> Response {
    let deal = FullDeal::random_with_id(deal_id);
    (StatusCode::OK, Json(deal)).into_response()
}

pub async fn healthcheck_handler() -> Response {
    let resp_msg = serde_json::json!({
        "health_status": HealthCheckStatus::Green,
        "database": "healthy",
        "job_queue": {
            "active": 1,
            "errored": 23,
            "scheduled": 178,
        },
    });

    (StatusCode::OK, Json(resp_msg)).into_response()
}

/// Returns the accumulated metrics since the start of the billing period. Storage, bandwidth, and
/// sealed values are in bytes.
pub async fn metrics_current_handler() -> Response {
    let mut rng = rand::thread_rng();

    let used_storage = rng.gen_range(1099511627776u64..=2528876743884u64);
    let available_storage = used_storage + rng.gen_range(1099511627776u64..=2528876743884u64);

    let egress_bandwidth = rng.gen_range(0u64..=17179869184u64);
    let ingress_bandwidth = (egress_bandwidth as f32 * rng.gen_range(1.2f32..=2.3f32)) as usize;

    let resp_msg = serde_json::json!({
        "bandwidth": {
            "egress": egress_bandwidth,
            "ingress": ingress_bandwidth,
        },
        "deals": {
            "accepted": rng.gen_range(0..=99),
            "sealed": rng.gen_range(109951162u64..=252887674u64),
        },
        "storage": {
            "used": used_storage,
            "available": available_storage,
        },
    });

    (StatusCode::OK, Json(resp_msg)).into_response()
}

pub async fn metrics_bandwidth_daily_handler() -> Response {
    let mut readings = Vec::new();

    let current_time = OffsetDateTime::now_utc();
    let mut date = current_time.date() - 30i64.days();

    for _day in 0..30 {
        date = date.next_day().expect("tomorrow");
        readings.push(BandwidthMeasurement::random_with_date(date));
    }

    (StatusCode::OK, Json(readings)).into_response()
}

pub async fn metrics_storage_daily_handler() -> Response {
    let mut readings = Vec::new();

    let current_time = OffsetDateTime::now_utc();
    let mut date = current_time.date() - 30i64.days();

    for _day in 0..30 {
        date = date.next_day().expect("tomorrow");
        readings.push(StorageMeasurement::random_with_date(date));
    }

    (StatusCode::OK, Json(readings)).into_response()
}

async fn not_found_handler() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({"status": "not found"})),
    )
}

#[derive(Serialize)]
struct Alert {
    id: Uuid,

    #[serde(rename = "msg")]
    message: String,

    severity: AlertSeverity,
    details: AlertDetails,

    #[serde(with = "time::serde::rfc3339")]
    triggered_at: OffsetDateTime,

    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "time::serde::rfc3339::option"
    )]
    resolved_at: Option<OffsetDateTime>,
}

impl Alert {
    fn random() -> Self {
        let details = AlertDetails::SetupRequired;

        Self {
            id: Uuid::new_v4(),
            severity: AlertSeverity::Warning,
            message: details.to_string(),
            details,
            triggered_at: OffsetDateTime::now_utc(),
            resolved_at: None,
        }
    }

    fn random_resolved() -> Self {
        let details = AlertDetails::SetupRequired;

        Self {
            id: Uuid::new_v4(),
            severity: AlertSeverity::Warning,
            message: details.to_string(),
            details,
            triggered_at: OffsetDateTime::now_utc(),
            resolved_at: Some(OffsetDateTime::now_utc()),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
enum AlertDetails {
    AvailableDealExpired { id: Uuid },
    ProofFailed { id: Uuid },
    SetupRequired,
}

impl Display for AlertDetails {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let msg = match &self {
            AlertDetails::AvailableDealExpired { .. } => {
                "Available deal expired without being accepted!"
            }
            AlertDetails::ProofFailed { .. } => "A proof on sealed data failed to validate!",
            AlertDetails::SetupRequired => {
                "Additional configuration required before data will be stored"
            }
        };

        f.write_str(msg)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum AlertSeverity {
    Warning,
    Error,
    Fatal,
}

#[derive(Serialize)]
struct AvailableDeal {
    id: Uuid,

    /// Size of the deal in bytes
    size: usize,

    /// Price is in USD * 10_000, $2.45 would be 245000
    payment: usize,

    status: DealStatus,

    #[serde(with = "time::serde::rfc3339")]
    accept_by: OffsetDateTime,

    #[serde(with = "time::serde::rfc3339")]
    sealed_by: OffsetDateTime,
}

impl AvailableDeal {
    pub fn random() -> Self {
        Self::random_with_id(Uuid::new_v4())
    }

    pub fn random_with_id(id: Uuid) -> Self {
        let mut rng = rand::thread_rng();

        let size = rng.gen_range(1073741824..=30064771072);
        let payment = ((size * PRICE_PER_TIB) / (1024 * 1024 * 1024 * 1024)) * CURRENCY_MULTIPLIER;

        let future_offset = rng.gen_range(113_320..=233_280);
        let accept_by = OffsetDateTime::now_utc() + Duration::from_secs(future_offset);
        let sealed_by = accept_by + 1.days();

        Self {
            id,
            size,
            payment,
            status: DealStatus::Available,
            accept_by,
            sealed_by,
        }
    }
}

#[derive(Serialize)]
struct BandwidthMeasurement {
    date: Date,

    // value in bytes
    egress: u64,

    // value in bytes
    ingress: u64,
}

impl BandwidthMeasurement {
    pub fn random_with_date(date: Date) -> Self {
        let mut rng = rand::thread_rng();

        let egress = rng.gen_range(0u64..=171798691u64);
        let ingress = (egress as f32 * rng.gen_range(1.2f32..=2.3f32)) as u64;

        Self {
            date,
            egress,
            ingress,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum DealStatus {
    /// Initial state, the deal is available to be taken but has not been committed to
    Available,

    /// The deal has been accepted and the data is being assembled but is not ready to be sealed.
    Constructing,

    /// The deal has been accepted, and the data is available for the storage provider to seal the
    /// data.
    Pending,

    /// The deal reached the end of its accept_by window or was ignored
    NotAccepted,

    /// The deal was previously accepted, but was cancelled before sealing took place.
    Cancelled,

    /// The data was successfully sealed on to the network
    Sealed,

    /// We detected a proof violation in this deal
    Violated,

    /// The deal reached the end of its agreed term without renewal
    Completed,
}

impl DealStatus {
    pub fn random() -> Self {
        match rand::thread_rng().gen_range(1u8..=6u8) {
            1 => DealStatus::Constructing,
            2 => DealStatus::Pending,
            3 => DealStatus::Cancelled,
            4 => DealStatus::Sealed,
            5 => DealStatus::Violated,
            6 => DealStatus::Completed,
            _ => unreachable!(),
        }
    }
}

#[derive(Serialize)]
struct FullDeal {
    id: Uuid,

    /// Size of the deal in bytes
    size: usize,

    /// Price is in USD * 10_000, $2.45 would be 245000
    payment: usize,

    status: DealStatus,

    /// Not present in Pending state, time the deal was accepted by the user
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "time::serde::rfc3339::option"
    )]
    accepted_at: Option<OffsetDateTime>,

    /// Only present in the Cancelled state
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "time::serde::rfc3339::option"
    )]
    cancelled_at: Option<OffsetDateTime>,

    /// When the data needs to be sealed by (the deadline)
    #[serde(with = "time::serde::rfc3339")]
    sealed_by: OffsetDateTime,

    /// When the data was ACTUALLY sealed
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "time::serde::rfc3339::option"
    )]
    sealed_at: Option<OffsetDateTime>,

    /// When the sealed contract will end if not renewed
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "time::serde::rfc3339::option"
    )]
    completes_at: Option<OffsetDateTime>,
}

impl FullDeal {
    pub fn random() -> Self {
        Self::random_with_id(Uuid::new_v4())
    }

    pub fn random_with_id(id: Uuid) -> Self {
        let mut rng = rand::thread_rng();

        let size = rng.gen_range(1073741824..=30064771072);
        let payment = (size * PRICE_PER_TIB * CURRENCY_MULTIPLIER) / (1024 * 1024 * 1024 * 1024);

        let current_time = OffsetDateTime::now_utc();
        let past_offset = rng.gen_range(113_320i64..=233_280i64);
        let created_at = current_time - past_offset.seconds();

        let sealed_by = created_at + 1i64.days();

        let status = DealStatus::random();

        let cancelled_at = if matches!(status, DealStatus::Cancelled) {
            Some(current_time - rng.gen_range(20i64..=280i64).minutes())
        } else {
            None
        };

        let accepted_at = if matches!(status, DealStatus::Available | DealStatus::NotAccepted) {
            None
        } else {
            let aa = created_at + rng.gen_range(900i64..=86_400).seconds();
            Some(aa)
        };

        let (sealed_at, completes_at) = if matches!(
            status,
            DealStatus::Sealed | DealStatus::Violated | DealStatus::Completed
        ) {
            let aa = accepted_at
                .as_ref()
                .cloned()
                .expect("present in these status");

            let sa = aa + rng.gen_range(3600i64..7200i64).seconds();
            let ca = sa + 180i64.days();

            (Some(sa), Some(ca))
        } else {
            (None, None)
        };

        Self {
            id,
            size,
            payment,
            status,
            accepted_at,
            cancelled_at,
            sealed_by,
            sealed_at,
            completes_at,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum HealthCheckStatus {
    Red,
    Yellow,
    Green,
}

#[derive(Serialize)]
struct StorageMeasurement {
    date: Date,

    // value in bytes
    used: u64,

    // value in bytes
    available: u64,
}

impl StorageMeasurement {
    pub fn random_with_date(date: Date) -> Self {
        let mut rng = rand::thread_rng();

        let used = rng.gen_range(0u64..=171798691u64);
        let available = (used as f32 * rng.gen_range(1.2f32..=2.3f32)) as u64;

        Self {
            date,
            used,
            available,
        }
    }
}
