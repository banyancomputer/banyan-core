use axum::extract::{Json, State};
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use jwt_simple::prelude::ES384PublicKey;

use crate::api::models::ApiEscrowedKeyMaterial;
use crate::app::AppState;
use crate::extractors::SessionIdentity;
use crate::utils::keys::fingerprint_public_key;

pub async fn handler(
    session_id: SessionIdentity,
    State(state): State<AppState>,
    Json(request): Json<ApiEscrowedKeyMaterial>,
) -> Result<Response, CreateEscrowedDeviceError> {
    let api_public_key_pem = request.api_public_key_pem;
    let encryption_public_key_pem = request.encryption_public_key_pem;

    // Validate that the public key material is valid
    let public_device_api_key = ES384PublicKey::from_pem(&api_public_key_pem)
        .map_err(CreateEscrowedDeviceError::InvalidPublicKey)?;
    let _public_encryption_key = ES384PublicKey::from_pem(&encryption_public_key_pem)
        .map_err(CreateEscrowedDeviceError::InvalidPublicKey)?;
    let device_api_key_fingerprint = fingerprint_public_key(&public_device_api_key);

    let user_id = session_id.user_id().to_string();
    let encrypted_private_key_material = request.encrypted_private_key_material;
    let pass_key_salt = request.pass_key_salt;

    let database = state.database();
    let mut conn = database.begin().await?;

    let existing_device_key = sqlx::query_scalar!(
        "SELECT id FROM escrowed_devices WHERE user_id = $1",
        user_id,
    )
    .fetch_optional(&mut *conn)
    .await?;

    if existing_device_key.is_some() {
        return Err(CreateEscrowedDeviceError::EscrowedDeviceAlreadyExists);
    };

    sqlx::query!(
        r#"
            INSERT INTO escrowed_devices (
                user_id, 
                api_public_key_pem, 
                encryption_public_key_pem, 
                encrypted_private_key_material, 
                pass_key_salt
            )
            VALUES ($1, $2, $3, $4, $5);
        "#,
        user_id,
        api_public_key_pem,
        encryption_public_key_pem,
        encrypted_private_key_material,
        pass_key_salt
    )
    .execute(&mut *conn)
    .await?;

    sqlx::query!(
        r#"
            INSERT INTO user_keys (name, user_id, fingerprint, pem, api_access)
            VALUES ($1, $2, $3, $4, TRUE);
        "#,
        "Owner",
        user_id,
        device_api_key_fingerprint,
        api_public_key_pem,
    )
    .execute(&mut *conn)
    .await?;

    conn.commit().await?;

    Ok((StatusCode::OK, ()).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum CreateEscrowedDeviceError {
    #[error("escrow device already exists for user")]
    EscrowedDeviceAlreadyExists,

    #[error("failed to create escrow device: {0}")]
    FailedToCreateEscrowedDevice(sqlx::Error),

    #[error("provided public key was not a valid EC P384 pem: {0}")]
    InvalidPublicKey(jwt_simple::Error),
}

impl From<sqlx::Error> for CreateEscrowedDeviceError {
    fn from(err: sqlx::Error) -> Self {
        CreateEscrowedDeviceError::FailedToCreateEscrowedDevice(err)
    }
}

impl IntoResponse for CreateEscrowedDeviceError {
    fn into_response(self) -> Response {
        use CreateEscrowedDeviceError as CEDE;
        match &self {
            CEDE::EscrowedDeviceAlreadyExists => {
                let err_msg = serde_json::json!({"msg": "escrow device already exists for user"});
                (StatusCode::CONFLICT, Json(err_msg)).into_response()
            }
            CEDE::InvalidPublicKey(_) => {
                let err_msg = serde_json::json!({"msg": "provided public key was not valid"});
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            _ => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the &request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
