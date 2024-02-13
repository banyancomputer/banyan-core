use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::app::{AppState, SerdeVersion};
use crate::extractors::StorageProviderIdentity;

pub async fn handler(
    storage_provider: StorageProviderIdentity,
    State(state): State<AppState>,
    Json(version): Json<SerdeVersion>,
) -> Result<Response, ReportHealthHookError> {
    let database = state.database();
    let mut conn = database.begin().await?;

    let current_version = version.version;
    let storage_provider_id = storage_provider.id;

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
pub enum ReportHealthHookError {
    #[error("sql error: {0}")]
    SqlxError(#[from] sqlx::Error),
}

impl IntoResponse for ReportHealthHookError {
    fn into_response(self) -> Response {
        tracing::error!("{self}");
        let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
    }
}
