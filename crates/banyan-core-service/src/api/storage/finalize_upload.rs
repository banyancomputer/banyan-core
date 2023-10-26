use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use uuid::Uuid;

use crate::app::AppState;
use crate::extractors::StorageProviderIdentity;

/// When a client finishes uploading their data to either staging or a storage host, the storage
/// host will make a request to this end point letting us know that we have all the data safely
/// stored and can mark the associated metadata as ready to be consumed by downstream clients.
#[axum::debug_handler]
pub async fn handler(
    storage_provider: StorageProviderIdentity,
    State(state): State<AppState>,
    Path(metadata_id): Path<Uuid>,
    Json(request): Json<FinalizeUploadRequest>,
) -> Result<Response, FinalizeUploadError> {
    let db_data_size = request.data_size;
    let db_metadata_id = metadata_id.to_string();

    let database = state.database();

    sqlx::query!(
        r#"UPDATE storage_grants
               SET redeemed_at = CURRENT_TIMESTAMP
               WHERE storage_host_id = $1
                   AND id = $2
                   AND redeemed_at IS NULL;"#,
        storage_provider.id,
        request.storage_authorization_id,
    )
    .execute(&database)
    .await
    .map_err(FinalizeUploadError::RedeemFailed)?;

    sqlx::query!(
        r#"INSERT INTO storage_hosts_metadatas_storage_grants
               (storage_host_id, metadata_id, storage_grant_id)
               VALUES ($1, $2, $3);"#,
        storage_provider.id,
        db_metadata_id,
        request.storage_authorization_id,
    )
    .execute(&database)
    .await
    .map_err(FinalizeUploadError::NoUploadAssociation)?;

    for block_cid in request.normalized_cids.iter() {
        sqlx::query!("INSERT OR IGNORE INTO blocks (cid) VALUES ($1);", block_cid)
            .execute(&database)
            .await
            .map_err(FinalizeUploadError::UnableToRecordBlock)?;

        let block_id = sqlx::query_scalar!("SELECT id FROM blocks WHERE cid = $1", block_cid)
            .fetch_one(&database)
            .await
            .map_err(FinalizeUploadError::UnableToRecordBlock)?;

        // Completeley insert the block location into the database, treating it like we've definitely never seen it before
        sqlx::query!(
            r#"INSERT INTO block_locations
            (block_id, metadata_id, storage_host_id)
            VALUES ($1, $2, $3);"#,
            block_id,
            db_metadata_id,
            storage_provider.id,
        )
        .execute(&database)
        .await
        .map_err(FinalizeUploadError::UnableToRecordBlock)?;
    }

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
    )
    .execute(&database)
    .await
    .map_err(FinalizeUploadError::MarkOutdatedFailed)?;

    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

#[derive(Deserialize)]
pub struct FinalizeUploadRequest {
    data_size: i64,
    normalized_cids: Vec<String>,
    storage_authorization_id: String,
}

#[derive(Debug, thiserror::Error)]
pub enum FinalizeUploadError {
    #[error("failed to mark the completed upload as current: {0}")]
    MarkCurrentFailed(sqlx::Error),

    #[error("failed to existing current upload(s) as outdated: {0}")]
    MarkOutdatedFailed(sqlx::Error),

    #[error("failed to associate finalized uploaded with storage host")]
    NoUploadAssociation(sqlx::Error),

    #[error("failed to register storage grant as redeemed: {0}")]
    RedeemFailed(sqlx::Error),

    #[error("error occurred while recording a blocks present: {0}")]
    UnableToRecordBlock(sqlx::Error),
}

impl IntoResponse for FinalizeUploadError {
    fn into_response(self) -> Response {
        tracing::error!("{self}");
        let err_msg = serde_json::json!({"msg": "internal server error"});
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
    }
}
