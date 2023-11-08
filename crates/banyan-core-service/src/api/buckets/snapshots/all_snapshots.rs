use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::api::models::ApiSnapshot;
use crate::app::AppState;
use crate::database::models::Snapshot;
use crate::extractors::UserIdentity;

pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
    Path(bucket_id): Path<Uuid>,
) -> Result<Response, AllSnapshotsError> {
    let database = state.database();
    let bucket_id = bucket_id.to_string();

    let user_id = user_identity.id().to_string();
    let query_result = sqlx::query_as!(
        Snapshot,
        "SELECT s.* FROM snapshots AS s
             JOIN metadata AS m ON s.metadata_id = m.id
             JOIN buckets AS b ON m.bucket_id = b.id
             WHERE b.user_id = $1 AND m.bucket_id = $2;",
        user_id,
        bucket_id,
    )
    .fetch_all(&database)
    .await
    .map_err(AllSnapshotsError::DatabaseFailure)?;

    let snapshots: Vec<_> = query_result.into_iter().map(ApiSnapshot::from).collect();

    Ok((StatusCode::OK, Json(snapshots)).into_response())
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
