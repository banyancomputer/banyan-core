use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::app::{AppState, Version};
use crate::extractors::StorageProviderIdentity;

pub async fn handler(
    State(state): State<AppState>,
    storage_provider_id: StorageProviderIdentity,
    Json(version): Json<Version>,
) -> Result<Response, HealthCheckHookError> {
    let database = state.database();
    let mut conn = database.begin().await?;

    let current_version = version.version;
    let storage_provider_id = storage_provider_id.id;

    sqlx::query!(
        r#"
            UPDATE storage_hosts
            SET last_seen_at = CURRENT_TIMESTAMP, current_version = $1
            WHERE id = $2;
        "#,
        current_version,
        storage_provider_id,
    )
    .execute(&mut *conn)
    .await?;

    conn.commit().await?;

    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum HealthCheckHookError {
    #[error("sql error: {0}")]
    SqlxError(#[from] sqlx::Error),
}

impl IntoResponse for HealthCheckHookError {
    fn into_response(self) -> Response {
        tracing::error!("{self}");
        let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
    }
}
