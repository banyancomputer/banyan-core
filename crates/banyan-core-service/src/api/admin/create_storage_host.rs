use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use jwt_simple::prelude::*;

use crate::api::models::ApiSelectedStorageHostAdmin;
use crate::app::AppState;
use crate::extractors::AdminIdentity;
use crate::utils::keys::fingerprint_public_key;

#[derive(Serialize, Deserialize)]
pub struct SelectedStorageHostRequest {
    pub name: String,
    pub url: String,
    pub available_storage: i64,
    pub region: String,
}

pub async fn handler(
    _: AdminIdentity,
    State(state): State<AppState>,
    Json(request): Json<SelectedStorageHostRequest>,
) -> Result<Response, CreateStorageHostError> {
    let new_key = ES384KeyPair::generate();
    let fingerprint = fingerprint_public_key(&new_key.public_key());
    let pem = match new_key.public_key().to_pem() {
        Ok(key) => key,
        Err(_err) => return Err(CreateStorageHostError::CouldNotRecoveryPublicKey),
    };

    let database = state.database();
    let storage_host_id = sqlx::query_scalar!(
        r#"INSERT INTO storage_hosts (name, url, used_storage, available_storage, region, fingerprint, pem) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING id;"#,
        request.name,
        request.url,
        0,
        request.available_storage,
        request.region,
        fingerprint,
        pem
    )
    .fetch_one(&database)
    .await
    .map_err(CreateStorageHostError::Database)?;

    Ok((
        StatusCode::OK,
        Json(ApiSelectedStorageHostAdmin {
            id: storage_host_id,
            name: request.name,
            url: request.url,
            used_storage: 0,
            available_storage: request.available_storage,
            fingerprint,
            pem,
        }),
    )
        .into_response())
}

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum CreateStorageHostError {
    #[error("failed insertion: {0}")]
    Database(#[from] sqlx::Error),
    #[error("could not create public key")]
    CouldNotRecoveryPublicKey,
}
impl IntoResponse for CreateStorageHostError {
    fn into_response(self) -> Response {
        tracing::error!("failed to lookup all deals: {self}");
        let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
    }
}
