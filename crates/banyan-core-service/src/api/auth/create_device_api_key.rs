use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use jwt_simple::prelude::*;
use serde::Deserialize;

use crate::app::AppState;
use crate::extractors::UserIdentity;
use crate::utils::keys::sha1_fingerprint_publickey;

/// Register a new device api key with an account
pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
    Json(request): Json<CreateDeviceApiKeyRequest>,
) -> Result<Response, CreateDeviceApiKeyError> {
    let public_device_key = ES384PublicKey::from_pem(&request.pem)
        .map_err(CreateDeviceApiKeyError::InvalidPublicKey)?;

    let database = state.database();
    let fingerprint = sha1_fingerprint_publickey(&public_device_key);

    let user_id = user_identity.id().to_string();
    let device_api_key_id = sqlx::query_scalar!(
        r#"INSERT INTO device_api_keys (user_id, fingerprint, pem)
            VALUES ($1, $2, $3)
            RETURNING id;"#,
        user_id,
        fingerprint,
        request.pem,
    )
    .fetch_one(&database)
    .await
    .map_err(CreateDeviceApiKeyError::FailedToCreateKey)?;

    let resp_msg = serde_json::json!({"id": device_api_key_id, "fingerprint": fingerprint});
    Ok((StatusCode::OK, Json(resp_msg)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum CreateDeviceApiKeyError {
    #[error("failed to store device API key: {0}")]
    FailedToCreateKey(sqlx::Error),

    #[error("provided public key was not a valid EC P384 pem")]
    InvalidPublicKey(jwt_simple::Error),
}

impl IntoResponse for CreateDeviceApiKeyError {
    fn into_response(self) -> Response {
        match &self {
            CreateDeviceApiKeyError::InvalidPublicKey(_) => {
                let err_msg = serde_json::json!({"msg": "provided public key was not valid"});
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            _ => {
                tracing::error!("failed to create device api key: {self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}

#[derive(Deserialize)]
pub struct CreateDeviceApiKeyRequest {
    pem: String,
}
