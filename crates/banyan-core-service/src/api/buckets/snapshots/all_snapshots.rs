use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use uuid::Uuid;

use crate::app::AppState;
use crate::database::models::Snapshot;
use crate::extractors::ApiToken;
use crate::api::models::ApiSnapshot;

pub async fn handler(
    api_token: ApiToken,
    State(state): State<AppState>,
    Path(bucket_id): Path<Uuid>,
) -> Result<Response, AllSnapshotsError> {
    let database = state.database();
    let bucket_id = bucket_id.to_string();

    let query_result = sqlx::query_as!(
        Snapshot,
        "SELECT s.*,(m.data_size + m.metadata_size) AS size FROM snapshots AS s
             JOIN metadata AS m ON s.metadata_id = m.id
             JOIN buckets AS b ON m.bucket_id = b.id
             WHERE b.account_id = $1 AND m.bucket_id = $2;",
        api_token.subject,
        bucket_id,
    )
    .fetch_all(&database)
    .await
    .map_err(AllSnapshotsError::DatabaseFailure)?;

    let buckets: Vec<_> = query_result.into_iter().map(|db| ApiSnapshot::from(db)).collect();

    Ok((StatusCode::OK, Json(buckets)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum AllSnapshotsError {
    #[error("failed to query the database: {0}")]
    DatabaseFailure(sqlx::Error),
}

impl IntoResponse for AllSnapshotsError {
    fn into_response(self) -> Response {
        tracing::error!("failed to lookup all snapshots: {self}");
        let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
    }
}
