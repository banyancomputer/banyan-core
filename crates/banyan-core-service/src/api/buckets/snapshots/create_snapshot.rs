use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::app::AppState;
use crate::extractors::ApiIdentity;

pub async fn handler(
    api_id: ApiIdentity,
    State(state): State<AppState>,
    Path((bucket_id, metadata_id)): Path<(Uuid, Uuid)>,
) -> Result<Response, CreateSnapshotError> {
    let database = state.database();

    let bucket_id = bucket_id.to_string();
    let metadata_id = metadata_id.to_string();

    let owned_metadata_id = sqlx::query_scalar!(
        r#"SELECT m.id FROM metadata AS m
               JOIN buckets AS b ON m.bucket_id = b.id
               LEFT JOIN snapshots AS s ON s.metadata_id = m.id
               WHERE b.user_id = $1
                   AND b.id = $2
                   AND m.id = $3
                   AND s.id IS NULL;"#,
        api_id.user_id,
        bucket_id,
        metadata_id,
    )
    .fetch_optional(&database)
    .await
    .map_err(CreateSnapshotError::MetadataUnavailable)?
    .ok_or(CreateSnapshotError::NotFound)?;

    let snapshot_id = sqlx::query_scalar!(
        r#"INSERT INTO snapshots (metadata_id, state)
               VALUES ($1, 'pending')
               RETURNING id;"#,
        owned_metadata_id,
    )
    .fetch_one(&database)
    .await
    .map_err(CreateSnapshotError::SaveFailed)?;

    let resp_msg = serde_json::json!({ "id": snapshot_id });
    Ok((StatusCode::OK, Json(resp_msg)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum CreateSnapshotError {
    #[error("no matching metadata for the current account")]
    NotFound,

    #[error("unable to locate requested metadata: {0}")]
    MetadataUnavailable(sqlx::Error),

    #[error("saving new snapshot association failed")]
    SaveFailed(sqlx::Error),
}

impl IntoResponse for CreateSnapshotError {
    fn into_response(self) -> Response {
        match &self {
            CreateSnapshotError::NotFound => {
                let err_msg = serde_json::json!({"msg": "not found"});
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            }
            _ => {
                tracing::error!("encountered error creating snapshot: {self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
