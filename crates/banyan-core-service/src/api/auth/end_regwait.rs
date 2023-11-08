use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::api::auth::registration_event::RegistrationEvent;
use crate::app::AppState;
use crate::event_bus::{EventBusError, SystemEvent};
use crate::extractors::UserIdentity;

pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
    Path(fingerprint): Path<String>,
) -> Result<Response, EndRegwaitError> {
    let database = state.database();

    let user_id = user_identity.id().to_string();
    let maybe_present = sqlx::query_scalar!(
        r#"SELECT 1 FROM device_api_keys
               WHERE user_id = $1
                   AND fingerprint = $2;"#,
        user_id,
        fingerprint,
    )
    .fetch_optional(&database)
    .await
    .map_err(EndRegwaitError::LookupFailed)?;

    let registration_event = match maybe_present {
        Some(_) => RegistrationEvent::approved(fingerprint, user_id),
        None => RegistrationEvent::rejected(fingerprint),
    };

    state
        .event_bus()
        .send(SystemEvent::DeviceKeyRegistration, &registration_event)
        .map_err(EndRegwaitError::NoAnnouncement)?;

    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum EndRegwaitError {
    #[error("failed to lookup the key: {0}")]
    LookupFailed(sqlx::Error),

    #[error("failed to announce a device registration on the bus: {0}")]
    NoAnnouncement(EventBusError),
}

impl IntoResponse for EndRegwaitError {
    fn into_response(self) -> Response {
        tracing::error!("{self}");
        let err_msg = serde_json::json!({"msg": "an internal service issue occurred"});
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
    }
}
