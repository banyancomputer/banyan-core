use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use uuid::Uuid;

use crate::app::AppState;
use crate::database::Database;
use crate::extractors::StorageHostToken;

/// When a client finishes uploading their data to either staging or a storage host, the storage
/// host will make a request to this end point letting us know that we have all the data safely
/// stored and can mark the associated metadata as ready to be consumed by downstream clients.
#[axum::debug_handler]
pub async fn handler(
    _storage_host_token: StorageHostToken,
    //database: Database, // todo: needs a fromrequestparts...
    State(state): State<AppState>,
    Path(metadata_id): Path<Uuid>,
    Json(request): Json<FinalizeUploadRequest>,
) -> Result<Response, FinalizeUploadError> {
    let db_data_size = request.data_size as i64;
    let db_metadata_id = metadata_id.to_string();

    let database = state.database();

    let bucket_id = sqlx::query_scalar!(
        r#"UPDATE metadata
             SET state = 'current' AND data_size = $1
             WHERE id = $2 AND state = 'pending'
             RETURNING bucket_id;"#,
        db_data_size,
        db_metadata_id,
    )
    .fetch_one(&database)
    .await
    .map_err(FinalizeUploadError::MarkCurrentFailed)?;

    // Downgrade other metadata for this bucket to outdated if they were in current state
    // Except for the metadata that was just updated
    sqlx::query!(
        r#"UPDATE metadata
             SET state = 'outdated'
             WHERE bucket_id = $1 AND state = 'current' AND id != $2;"#,
        bucket_id,
        db_metadata_id,
    ).execute(&database).await.map_err(FinalizeUploadError::MarkOutdatedFailed)?;

    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

#[derive(Deserialize)]
pub struct FinalizeUploadRequest {
    data_size: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum FinalizeUploadError {
    #[error("failed to mark the completed upload as current: {0}")]
    MarkCurrentFailed(sqlx::Error),

    #[error("failed to existing current upload(s) as outdated: {0}")]
    MarkOutdatedFailed(sqlx::Error),
}

impl IntoResponse for FinalizeUploadError {
    fn into_response(self) -> Response {
        tracing::error!("{self}");
        let err_msg = serde_json::json!({"msg": "internal server error"});
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
    }
}
