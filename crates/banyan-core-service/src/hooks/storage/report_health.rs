use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::app::AppState;
use crate::extractors::StorageProviderIdentity;

#[derive(Deserialize, Serialize)]
pub(crate) struct ReportHealth {
    pub build_profile: String,
    pub features: Vec<String>,
    pub version: String,
}

pub async fn handler(
    storage_provider: StorageProviderIdentity,
    State(state): State<AppState>,
    Json(report): Json<ReportHealth>,
) -> Result<Response, ReportHealthHookError> {
    let database = state.database();
    let mut conn = database.begin().await?;

    let current_time = OffsetDateTime::now_utc();
    let current_version = report.version;
    let storage_provider_id = storage_provider.id;

    sqlx::query!(
        r#"
            UPDATE storage_hosts
            SET last_seen_at = $1, current_version = $2
            WHERE id = $3;
        "#,
        current_time,
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
