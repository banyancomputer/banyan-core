use crate::app::AppState;
use crate::utils::keys::fingerprint_public_key;
use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use jwt_simple::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateFakeUserRequest {
    device_api_key_pem: String,
}

/// Create a fake account for testing purposes -- bypasses oauth
#[axum::debug_handler]
pub async fn handler(
    State(state): State<AppState>,
    Json(request): Json<CreateFakeUserRequest>,
) -> Result<Response, CreateFakeUserError> {
    let public_key = ES384PublicKey::from_pem(&request.device_api_key_pem)
        .map_err(CreateFakeUserError::InvalidPublicKey)?;

    let database = state.database();
    let fingerprint = fingerprint_public_key(&public_key);

    let email = format!("{fingerprint}@user.com");
    let display_name = "fake_user";
    let user_id = sqlx::query_scalar!(
        r#"INSERT INTO users (display_name, email) VALUES ($1, $2) RETURNING id;"#,
        display_name,
        email
    )
    .fetch_one(&database)
    .await
    .map_err(CreateFakeUserError::UserCreationFailed)?;

    sqlx::query!(
        "INSERT INTO device_api_keys (user_id, fingerprint, pem) VALUES ($1, $2, $3);",
        user_id,
        fingerprint,
        request.device_api_key_pem,
    )
    .execute(&database)
    .await
    .map_err(CreateFakeUserError::ApiKeyCreationFailed)?;

    let response = serde_json::json!({"id": user_id});
    Ok((StatusCode::OK, Json(response)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum CreateFakeUserError {
    #[error("failed to create API key for dummy use: {0}")]
    ApiKeyCreationFailed(sqlx::Error),

    #[error("provided public key was not a valid P384 EC public key: {0}")]
    InvalidPublicKey(jwt_simple::Error),

    #[error("failed to create a new dummy user: {0}")]
    UserCreationFailed(sqlx::Error),
}

impl IntoResponse for CreateFakeUserError {
    fn into_response(self) -> Response {
        use CreateFakeUserError as CFAE;

        match self {
            CFAE::InvalidPublicKey(_) => {
                let err_msg = serde_json::json!({"msg": self.to_string()});
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            _ => {
                tracing::error!("{self}");
                let err_msg =
                    serde_json::json!({"msg": "backend service issue prevented account creation"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
