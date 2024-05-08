use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use jwt_simple::prelude::*;
use serde::Deserialize;

use crate::app::AppState;
use crate::extractors::UserIdentity;
use crate::utils::keys::fingerprint_public_key;

/// Register a new device api key with an account
pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
    Json(request): Json<CreateUserKeyRequest>,
) -> Result<Response, CreateUserKeyError> {
    let public_device_key =
        ES384PublicKey::from_pem(&request.pem).map_err(CreateUserKeyError::InvalidPublicKey)?;
    let database = state.database();
    let fingerprint = fingerprint_public_key(&public_device_key);
    let user_id = user_identity.id().to_string();
    let user_key_id = sqlx::query_scalar!(
        r#"
            INSERT INTO user_keys (name, user_id, fingerprint, pem, api_access)
            VALUES ($1, $2, $3, $4, TRUE)
            RETURNING id;
        "#,
        request.name,
        user_id,
        fingerprint,
        request.pem,
    )
    .fetch_one(&database)
    .await
    .map_err(CreateUserKeyError::FailedToCreateKey)?;

    let resp_msg = serde_json::json!({"id": user_key_id, "fingerprint": fingerprint});
    Ok((StatusCode::OK, Json(resp_msg)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum CreateUserKeyError {
    #[error("failed to store device API key: {0}")]
    FailedToCreateKey(sqlx::Error),

    #[error("provided public key was not a valid EC P384 pem")]
    InvalidPublicKey(jwt_simple::Error),
}

impl IntoResponse for CreateUserKeyError {
    fn into_response(self) -> Response {
        match &self {
            CreateUserKeyError::InvalidPublicKey(_) => {
                let err_msg = serde_json::json!({"msg": "provided public key was not valid"});
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            _ => {
                tracing::error!("failed to create user key: {self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}

#[derive(Deserialize)]
pub struct CreateUserKeyRequest {
    name: String,
    pem: String,
}
