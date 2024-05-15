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
    // Extract the public key PEM
    let public_key = request.public_key;

    // Validate that the public key material is valid
    let public_user_key = ES384PublicKey::from_pem(&public_key)
        .map_err(CreateEscrowedDeviceError::InvalidPublicKey)?;
    let user_key_fingerprint = fingerprint_public_key(&public_user_key);

    let user_id = session_id.user_id().to_string();
    let encrypted_private_key_material = request.encrypted_private_key_material;
    let pass_key_salt = request.pass_key_salt;

    let database = state.database();
    let mut conn = database.begin().await?;

    let existing_user_key = sqlx::query_scalar!(
        "SELECT id FROM escrowed_user_keys WHERE user_id = $1",
        user_id,
    )
    .fetch_optional(&mut *conn)
    .await?;

    if existing_user_key.is_some() {
        return Err(CreateEscrowedDeviceError::EscrowedDeviceAlreadyExists);
    };

    sqlx::query!(
        r#"
            INSERT INTO escrowed_user_keys (
                user_id, 
                public_key, 
                encrypted_private_key_material, 
                pass_key_salt
            )
            VALUES ($1, $2, $3, $4);
        "#,
        user_id,
        public_key,
        encrypted_private_key_material,
        pass_key_salt
    )
    .execute(&mut *conn)
    .await?;

    sqlx::query!(
        r#"
            INSERT INTO user_keys (name, user_id, fingerprint, public_key, api_access)
            VALUES ($1, $2, $3, $4, TRUE);
        "#,
        "Owner",
        user_id,
        user_key_fingerprint,
        public_key,
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
