use std::fmt::Display;
use std::time::Duration;

use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use http::{HeaderMap, HeaderValue};
use rand::Rng;
use serde::Serialize;
use time::ext::NumericalDuration;
use time::{Date, OffsetDateTime};
use uuid::Uuid;

use crate::app::AppState;

const CURRENCY_MULTIPLIER: usize = 10_000;

const PRICE_PER_TIB: usize = 2 * CURRENCY_MULTIPLIER;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(deal_all_handler))
        .route("/:deal_id", get(deal_single_handler))
        .route("/:deal_id/accept", get(deal_accept_handler))
        .route("/:deal_id/cancel", get(deal_cancel_handler))
        .route("/:deal_id/download", get(deal_download_handler))
        .route("/:deal_id/ignore", get(deal_ignore_handler))
        .route("/:deal_id/proof", get(deal_proof_handler))
        .with_state(state)
}

pub async fn deal_accept_handler(Path(_deal_id): Path<Uuid>) -> Response {
    (StatusCode::NO_CONTENT, ()).into_response()
}

pub async fn deal_all_handler() -> Response {
    let mut rng = rand::thread_rng();
    let target_count = rng.gen_range(3..=15);
    let mut deals = Vec::new();

    while deals.len() < target_count {
        let new_deal = FullDeal::random();

        if matches!(new_deal.status, DealStatus::Available) {
            continue;
        }

        deals.push(new_deal);
    }

    (StatusCode::OK, Json(deals)).into_response()
}

pub async fn deal_available_handler() -> Response {
    let deals = vec![
        AvailableDeal::random(),
        AvailableDeal::random(),
        AvailableDeal::random(),
        AvailableDeal::random(),
    ];

    (StatusCode::OK, Json(deals)).into_response()
}

pub async fn deal_cancel_handler(Path(_deal_id): Path<Uuid>) -> Response {
    (StatusCode::NO_CONTENT, ()).into_response()
}

/// Note: this only allows downloading of files in the Pending state
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
