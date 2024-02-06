use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use rand::Rng;
use serde::Serialize;
use time::ext::NumericalDuration;
use time::{Date, OffsetDateTime};

use crate::app::AppState;

const CURRENCY_MULTIPLIER: usize = 10_000;

const PRICE_PER_TIB: usize = 2 * CURRENCY_MULTIPLIER;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/current", get(metrics_current_handler))
        .route("/bandwidth/daily", get(metrics_bandwidth_daily_handler))
        .route("/storage/daily", get(metrics_storage_daily_handler))
        .with_state(state)
}

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
