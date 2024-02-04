use std::fmt::{self, Display, Formatter};

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::app::AppState;

const CURRENCY_MULTIPLIER: usize = 10_000;

const PRICE_PER_TIB: usize = 2 * CURRENCY_MULTIPLIER;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(alerts_handler))
        .route("/history", get(alert_history_handler))
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
