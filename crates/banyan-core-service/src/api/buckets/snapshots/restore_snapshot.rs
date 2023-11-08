use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::app::AppState;
use crate::extractors::UserIdentity;

pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
    Path((bucket_id, metadata_id)): Path<(Uuid, Uuid)>,
) -> Result<Response, RestoreSnapshotError> {
    let database = state.database();

    let bucket_id = bucket_id.to_string();
    let metadata_id = metadata_id.to_string();

    let user_id = user_identity.id().to_string();
    let snapshot_id = sqlx::query_scalar!(
        r#"SELECT s.id FROM snapshots AS s
               JOIN metadata AS m ON s.metadata_id = m.id
               JOIN buckets AS b ON m.bucket_id = b.id
               WHERE b.user_id = $1
                   AND b.id = $2
                   AND m.id = $3;"#,
        user_id,
        bucket_id,
        metadata_id,
    )
    .fetch_optional(&database)
    .await
    .map_err(RestoreSnapshotError::SnapshotUnavailable)?
    .ok_or(RestoreSnapshotError::NotFound)?;

    let request_id = sqlx::query_scalar!(
        r#"INSERT INTO snapshot_restore_requests (user_id, snapshot_id, state)
               VALUES ($1, $2, 'pending')
               RETURNING id;"#,
        user_id,
        snapshot_id,
    )
    .fetch_one(&database)
    .await
    .map_err(RestoreSnapshotError::FailedRequestGeneration)?;

    let resp_msg = serde_json::json!({ "id": request_id });
    Ok((StatusCode::OK, Json(resp_msg)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum RestoreSnapshotError {
    #[error("failed to store request for restoration in the database")]
    FailedRequestGeneration(sqlx::Error),

    #[error("no matching metadata for the current account")]
    NotFound,

    #[error("unable to locate requested snapshot: {0}")]
    SnapshotUnavailable(sqlx::Error),
}

impl IntoResponse for RestoreSnapshotError {
    fn into_response(self) -> Response {
        match &self {
            RestoreSnapshotError::NotFound => {
                let err_msg = serde_json::json!({"msg": "not found"});
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            }
            _ => {
                tracing::error!("encountered error restoring snapshot: {self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
