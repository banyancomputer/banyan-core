use std::time::Duration;

use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tokio::time::timeout;

use crate::api::auth::registration_event::{RegistrationEvent, RegistrationEventStatus};
use crate::app::AppState;
use crate::event_bus::{EventBusReceiver, SystemEvent};

pub async fn handler(
    State(state): State<AppState>,
    Path(fingerprint): Path<String>,
) -> Result<Response, StartRegwaitError> {
    let event_bus = state.event_bus().subscribe();

    let registration_event = timeout(
        Duration::from_secs(3),
        watch_for_fingerprint(fingerprint, event_bus),
    )
    .await
    .map_err(|_| StartRegwaitError::TimedOut)??;

    match registration_event.status {
        RegistrationEventStatus::Approved(account_id) => {
            let resp_msg = serde_json::json!({"account_id": account_id});
            Ok((StatusCode::OK, Json(resp_msg)).into_response())
        }
        RegistrationEventStatus::Rejected => {
            let resp_msg = serde_json::json!({"msg": "device registration rejected"});
            Ok((StatusCode::UNAUTHORIZED, Json(resp_msg)).into_response())
        }
    }
}

async fn watch_for_fingerprint(
    fingerprint: String,
    mut bus_channel: EventBusReceiver,
) -> Result<RegistrationEvent, StartRegwaitError> {
    loop {
        let bus_event = bus_channel
            .recv()
            .await
            .map_err(StartRegwaitError::BrokenBus)?;

        match bus_event {
            (SystemEvent::DeviceKeyRegistration, data) => {
                let event: RegistrationEvent =
                    bincode::deserialize(&data[..]).map_err(StartRegwaitError::InvalidEvent)?;

                if event.fingerprint == fingerprint {
                    return Ok(event);
                }
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StartRegwaitError {
    #[error("error receiving announcments from the bus: {0}")]
    BrokenBus(tokio::sync::broadcast::error::RecvError),

    #[error("announced event did not match our expected schema: {0}")]
    InvalidEvent(bincode::Error),

    #[error("timed out waiting for registration event")]
    TimedOut,
}

impl IntoResponse for StartRegwaitError {
    fn into_response(self) -> Response {
        match &self {
            StartRegwaitError::TimedOut => {
                let err_msg = serde_json::json!({"msg": "device registration took too long"});
                (StatusCode::REQUEST_TIMEOUT, Json(err_msg)).into_response()
            }
            _ => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({"msg": "internal server error"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
