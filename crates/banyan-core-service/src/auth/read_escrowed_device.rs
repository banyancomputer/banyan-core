use axum::extract::{Json, State};
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use serde::Serialize;

use crate::app::AppState;
use crate::extractors::SessionIdentity;

pub async fn handler(
    session: Option<SessionIdentity>,
    State(state): State<AppState>,
) -> Result<Response, ReadEscrowedDeviceError> {
    let session = match session {
        Some(session) => session,
        None => return Err(ReadEscrowedDeviceError::Unauthorized),
    };

    let database = state.database();
    let user_id = session.user_id();

    let escrowed_device = sqlx::query_as!(
        EscrowedDevice,
        r#"SELECT api_public_key_pem, encryption_public_key_pem, encrypted_private_key_material, pass_key_salt
            FROM escrowed_devices
            WHERE user_id = $1;"#,
        user_id,
    )
    .fetch_one(&database)
    .await
    .map_err(ReadEscrowedDeviceError::FailedToReadEscrowedDevice)?;

    Ok((StatusCode::OK, Json(escrowed_device)).into_response())
}

#[derive(sqlx::FromRow, Serialize)]
struct EscrowedDevice {
    api_public_key_pem: String,
    encryption_public_key_pem: String,
    encrypted_private_key_material: String,
    pass_key_salt: String,
}

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum ReadEscrowedDeviceError {
    #[error("failed to create escrow device: {0}")]
    FailedToReadEscrowedDevice(sqlx::Error),
    #[error("request did not contain a valid session")]
    Unauthorized,
}

impl IntoResponse for ReadEscrowedDeviceError {
    fn into_response(self) -> Response {
        use ReadEscrowedDeviceError as CEDE;
        match &self {
            CEDE::Unauthorized => {
                let err_msg = serde_json::json!({"msg": "unauthorized"});
                (StatusCode::UNAUTHORIZED, Json(err_msg)).into_response()
            }
            _ => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
