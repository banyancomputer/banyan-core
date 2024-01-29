use axum::extract::{Json, State};
use axum::response::{IntoResponse, Response};
use http::StatusCode;

use crate::api::models::ApiEscrowedKeyMaterial;
use crate::app::AppState;
use crate::database::models::EscrowedDevice;
use crate::extractors::UserIdentity;

pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
) -> Result<Response, ReadEscrowedDeviceError> {
    let database = state.database();
    let user_id = user_identity.id().to_string();
    let escrowed_device = sqlx::query_as!(
        EscrowedDevice,
        r#"SELECT *
            FROM escrowed_devices WHERE user_id = $1;"#,
        user_id,
    )
    .fetch_one(&database)
    .await
    .map_err(ReadEscrowedDeviceError::UnableToReadEscrowedDevice)?;

    let api_escrowed_key_material = ApiEscrowedKeyMaterial::from(escrowed_device);
    Ok((StatusCode::OK, Json(api_escrowed_key_material)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum ReadEscrowedDeviceError {
    #[error("could not read user")]
    UnableToReadEscrowedDevice(sqlx::Error),
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
