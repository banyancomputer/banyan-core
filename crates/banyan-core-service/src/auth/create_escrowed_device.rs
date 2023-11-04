use axum::extract::{Json, State};
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use jwt_simple::prelude::ES384PublicKey;

use crate::app::AppState;
use crate::extractors::SessionIdentity;
use crate::utils::keys::sha1_fingerprint_publickey;
use crate::auth::escrowed_key_material::EscrowedKeyMaterial;

pub async fn handler(
    session: Option<SessionIdentity>,
    State(state): State<AppState>,
    Json(request): Json<EscrowedKeyMaterial>,
) -> Result<Response, CreateEscrowedDeviceError> {
    let session = match session {
        Some(session) => session,
        None => return Err(CreateEscrowedDeviceError::Unauthorized),
    };
    let api_public_key_pem = request.api_public_key_pem();
    let encryption_public_key_pem = request.encryption_public_key_pem();

    // Validate that the public key material is valid
    let public_device_api_key = ES384PublicKey::from_pem(&api_public_key_pem)
        .map_err(CreateEscrowedDeviceError::InvalidPublicKey)?;
    let _public_encryption_key = ES384PublicKey::from_pem(&encryption_public_key_pem)
        .map_err(CreateEscrowedDeviceError::InvalidPublicKey)?;
    let device_api_key_fingerprint = sha1_fingerprint_publickey(&public_device_api_key);
    // TODO: Validate the salt here too

    let database = state.database();
    let user_id = session.user_id();
    let encrypted_private_key_material = request.encrypted_private_key_material();
    let pass_key_salt = request.pass_key_salt();

    // Check if the user has an escrow device already
    if sqlx::query!(
        r#"SELECT id 
        FROM escrowed_devices 
        WHERE user_id = $1"#,
        user_id,
    )
    .fetch_optional(&database)
    .await?
    .is_some()
    {
        return Err(CreateEscrowedDeviceError::EscrowedDeviceAlreadyExists);
    };

    // Create the Escrowed Device
    let mut transaction = database.begin().await?;
    sqlx::query!(
        r#"INSERT INTO escrowed_devices (user_id, api_public_key_pem, encryption_public_key_pem, encrypted_private_key_material, pass_key_salt)
            VALUES ($1, $2, $3, $4, $5);"#,
        user_id,
        api_public_key_pem,
        encryption_public_key_pem,
        encrypted_private_key_material,
        pass_key_salt
    )
    .execute(&mut *transaction)
    .await
    .map_err(|err| {
        match err.as_database_error() {
            Some(db_err) => {
                if db_err.is_unique_violation() {
                    CreateEscrowedDeviceError::EscrowedDeviceAlreadyExists
                } else {
                    err.into()
                }
            }
            None => err.into()
        }
    })?;
    sqlx::query!(
        r#"INSERT INTO device_api_keys (user_id, fingerprint, pem)
            VALUES ($1, $2, $3);"#,
        user_id,
        device_api_key_fingerprint,
        api_public_key_pem,
    )
    .execute(&mut *transaction)
    .await
    .map_err(|err| match err.as_database_error() {
        Some(db_err) => {
            if db_err.is_unique_violation() {
                CreateEscrowedDeviceError::FingerprintAlreadyExists
            } else {
                err.into()
            }
        }
        None => err.into(),
    })?;
    transaction.commit().await?;

    // No Response OK
    Ok((StatusCode::OK, ()).into_response())
}

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum CreateEscrowedDeviceError {
    #[error("escrow device already exists for user")]
    EscrowedDeviceAlreadyExists,

    #[error("failed to create escrow device: {0}")]
    FailedToCreateEscrowedDevice(sqlx::Error),

    #[error("another device api key exists with the same fingerprint")]
    FingerprintAlreadyExists,

    #[error("provided public key was not a valid EC P384 pem")]
    InvalidPublicKey(jwt_simple::Error),

    #[error("request did not contain a valid session")]
    Unauthorized,
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
            CEDE::InvalidPublicKey(_) => {
                let err_msg = serde_json::json!({"msg": "provided public key was not valid"});
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            CEDE::EscrowedDeviceAlreadyExists => {
                let err_msg = serde_json::json!({"msg": "escrow device already exists for user"});
                (StatusCode::CONFLICT, Json(err_msg)).into_response()
            }
            CEDE::FingerprintAlreadyExists => {
                let err_msg = serde_json::json!({"msg": "another device api key exists with the same fingerprint"});
                (StatusCode::CONFLICT, Json(err_msg)).into_response()
            }
            CEDE::Unauthorized => {
                let err_msg = serde_json::json!({"msg": "unauthorized"});
                (StatusCode::UNAUTHORIZED, Json(err_msg)).into_response()
            }
            _ => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the &request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
