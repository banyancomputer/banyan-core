use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use futures::stream::StreamExt;
use jwt_simple::prelude::*;

use crate::api::admin::all_storage_hosts::AllStorageHostsError;
use crate::api::models::ApiSelectedStorageHostAdmin;
use crate::app::AppState;
use crate::extractors::AdminServiceIdentity;

#[derive(Serialize, Deserialize)]
pub struct SelectedStorageHostRequest {
    pub name: String,
    pub url: String,
    pub used_storage: i64,
    pub available_storage: i64,
    pub fingerprint: String,
    pub pem: String,
}
pub async fn handler(
    _: AdminServiceIdentity,
    State(state): State<AppState>,
    Json(request): Json<SelectedStorageHostRequest>,
) -> Result<Response, CreateStorageHostError> {
    let database = state.database();
    let storage_host_id = sqlx::query_scalar!(
        r#"INSERT INTO storage_hosts (name, url, used_storage, available_storage, fingerprint, pem) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id;"#,
        request.name,
        request.url,
        request.used_storage,
        request.available_storage,
        request.fingerprint,
        request.pem
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
            used_storage: request.used_storage,
            available_storage: request.available_storage,
            fingerprint: request.fingerprint,
            pem: request.pem,
        }),
    )
        .into_response())
}

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum CreateStorageHostError {
    #[error("failed insertion: {0}")]
    Database(#[from] sqlx::Error),
}
impl IntoResponse for CreateStorageHostError {
    fn into_response(self) -> Response {
        tracing::error!("failed to lookup all deals: {self}");
        let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
    }
}
