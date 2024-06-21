use axum::extract::{Json, State};
use axum::response::{IntoResponse, Response};
use http::StatusCode;

use crate::api::models::ApiEscrowedKeyMaterial;
use crate::app::AppState;
use crate::database::models::EscrowedDevice;
use crate::extractors::SessionIdentity;

pub async fn handler(
    session_id: SessionIdentity,
    State(state): State<AppState>,
) -> Result<Response, ReadEscrowedDeviceError> {
    let database = state.database();
    let user_id = session_id.user_id().to_string();

    let maybe_escrowed_device = sqlx::query_as!(
        EscrowedDevice,
        "SELECT * FROM escrowed_devices WHERE user_id = $1;",
        user_id,
    )
    .fetch_optional(&database)
    .await
    .map_err(ReadEscrowedDeviceError::DatabaseFailure)?;

    let escrowed_device = match maybe_escrowed_device {
        Some(ed) => ed,
        None => {
            let err_msg = serde_json::json!({"msg": "user does not have an escrowed key"});
            return Ok((StatusCode::NOT_FOUND, Json(err_msg)).into_response());
        }
    };

    let api_escrowed_key_material = ApiEscrowedKeyMaterial::from(escrowed_device);
    Ok((StatusCode::OK, Json(api_escrowed_key_material)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum ReadEscrowedDeviceError {
    #[error("database failure occurred looking up user's escrowed key: {0}")]
    DatabaseFailure(sqlx::Error),
}

impl IntoResponse for ReadEscrowedDeviceError {
    fn into_response(self) -> Response {
        {
            tracing::error!("encountered error reading user: {self}");
            let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
        }
    }
}
