use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use crate::app::AppState;
use crate::extractors::StorageProviderIdentity;

pub async fn handler(
    _: StorageProviderIdentity,
    State(state): State<AppState>,
    Json(request): Json<MeterTrafficRequest>,
) -> Result<Response, MeterTrafficError> {
    let database = state.database();
    let mut conn = database
        .acquire()
        .await
        .map_err(MeterTrafficError::FailedToStoreTrafficData)?;
    let fingerprint = request.fingerprint;
    let user_id: String = sqlx::query_scalar!(
        r#"SELECT user_id FROM device_api_keys WHERE fingerprint = $1"#,
        fingerprint,
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(MeterTrafficError::ClientNotFound)?;

    sqlx::query!(
        r#"INSERT INTO metrics_traffic (user_id, ingress, egress)
           VALUES ($1, $2, $3)"#,
        user_id,
        request.ingress,
        request.egress,
    )
    .execute(&mut *conn)
    .await
    .map_err(MeterTrafficError::FailedToStoreTrafficData)?;

    Ok((StatusCode::OK, ()).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum MeterTrafficError {
    #[error("failed to store traffic data: {0}")]
    FailedToStoreTrafficData(sqlx::Error),

    #[error("client not found: {0}")]
    ClientNotFound(sqlx::Error),
}

impl IntoResponse for MeterTrafficError {
    fn into_response(self) -> Response {
        match self {
            MeterTrafficError::ClientNotFound(_) => (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"msg": "client not found"})),
            )
                .into_response(),
            MeterTrafficError::FailedToStoreTrafficData(_) => {
                tracing::error!("failed to store traffic data: {:#?}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({"msg": "backend service experienced an issue servicing the request"}))).into_response()
            }
        }
    }
}

#[derive(Deserialize)]
pub struct MeterTrafficRequest {
    fingerprint: String,
    ingress: i64,
    egress: i64,
}
