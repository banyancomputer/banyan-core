use crate::app::AppState;
use crate::utils::keys::sha1_fingerprint_publickey;
use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use jwt_simple::prelude::*;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateFakeAccountRequest {
    device_api_key_pem: String,
}

/// Create a fake account for testing purposes -- bypasses oauth
#[axum::debug_handler]
pub async fn handler(
    State(state): State<AppState>,
    Json(request): Json<CreateFakeAccountRequest>,
) -> Result<Response, CreateFakeAccountError> {
    let public_key = ES384PublicKey::from_pem(&request.device_api_key_pem)
        .map_err(CreateFakeAccountError::InvalidPublicKey)?;

    let database = state.database();

    let user_id = sqlx::query_scalar!("INSERT INTO users DEFAULT VALUES RETURNING id;")
        .fetch_one(&database)
        .await
        .map_err(CreateFakeAccountError::UserCreationFailed)?;

    let account_id = sqlx::query_scalar!(
        r#"INSERT INTO accounts (userId, type, provider, providerAccountId)
                   VALUES ($1, "oauth", "not-google", 100033331337)
                   RETURNING id;"#,
        user_id,
    )
    .fetch_one(&database)
    .await
    .map_err(CreateFakeAccountError::AccountCreationFailed)?;

    let fingerprint = sha1_fingerprint_publickey(&public_key);

    tracing::info!(
        "inserting:\na.id: {}\nfingerprint: {}\nPEM: {}",
        account_id,
        fingerprint,
        request.device_api_key_pem
    );

    sqlx::query!(
        "INSERT INTO device_api_keys (account_id, fingerprint, pem) VALUES ($1, $2, $3);",
        account_id,
        fingerprint,
        request.device_api_key_pem,
    )
    .execute(&database)
    .await
    .map_err(CreateFakeAccountError::ApiKeyCreationFailed)?;

    let response = serde_json::json!({"id": account_id});
    Ok((StatusCode::OK, Json(response)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum CreateFakeAccountError {
    #[error("failed to create an account associated with dummy user: {0}")]
    AccountCreationFailed(sqlx::Error),

    #[error("failed to create API key for dummy use: {0}")]
    ApiKeyCreationFailed(sqlx::Error),

    #[error("provided public key was not a valid P384 EC public key: {0}")]
    InvalidPublicKey(jwt_simple::Error),

    #[error("failed to create a new dummy user: {0}")]
    UserCreationFailed(sqlx::Error),
}

impl IntoResponse for CreateFakeAccountError {
    fn into_response(self) -> Response {
        use CreateFakeAccountError as CFAE;

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
