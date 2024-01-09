use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use uuid::Uuid;

use crate::app::AppState;
use crate::database::models::Metadata;
use crate::database::DatabaseConnection;
use crate::extractors::StorageProviderIdentity;

/// When a client finishes uploading their data to either staging or a storage host, the storage
/// host will make a request to this end point letting us know that we have all the data safely
/// stored and can mark the associated metadata as ready to be consumed by downstream clients.
pub async fn handler(
    storage_provider: StorageProviderIdentity,
    State(state): State<AppState>,
    Path(metadata_id): Path<Uuid>,
    Json(request): Json<ReportUploadRequest>,
) -> Result<Response, ReportUploadError> {
    let db_metadata_id = metadata_id.to_string();

    let database = state.database();
    let mut db_conn = database.acquire().await?;

    redeem_storage_grant(
        &mut db_conn,
        &storage_provider.id,
        &request.storage_authorization_id,
    )
    .await?;
    associate_upload(
        &mut db_conn,
        &storage_provider.id,
        &db_metadata_id,
        &request.storage_authorization_id,
    )
    .await?;

    for block_cid in request.normalized_cids.iter() {
        sqlx::query!("INSERT OR IGNORE INTO blocks (cid) VALUES ($1);", block_cid)
            .execute(&mut *db_conn)
            .await
            .map_err(ReportUploadError::UnableToRecordBlock)?;

        let block_id = sqlx::query_scalar!("SELECT id FROM blocks WHERE cid = $1", block_cid)
            .fetch_one(&mut *db_conn)
            .await
            .map_err(ReportUploadError::UnableToRecordBlock)?;

        // Completeley insert the block location into the database, treating it like we've definitely never seen it before
        sqlx::query!(
            r#"INSERT INTO block_locations
            (block_id, metadata_id, storage_host_id)
            VALUES ($1, $2, $3);"#,
            block_id,
            db_metadata_id,
            storage_provider.id,
        )
        .execute(&mut *db_conn)
        .await
        .map_err(ReportUploadError::UnableToRecordBlock)?;
    }

    let bucket_id = Metadata::get_bucket_id(&mut db_conn, &db_metadata_id)
        .await
        .map_err(ReportUploadError::MarkCurrentFailed)?;

    // TODO: if a storage host is reporting for a piece of metadata in a bucket that has since
    // been soft-deleted, then this will return an error.
    // We should handle that invariant more explicitly.
    Metadata::mark_current(
        &mut db_conn,
        &bucket_id,
        &db_metadata_id,
        Some(request.data_size),
    )
    .await
    .map_err(ReportUploadError::MarkCurrentFailed)?;

    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

#[derive(Deserialize)]
pub struct ReportUploadRequest {
    data_size: i64,
    normalized_cids: Vec<String>,
    storage_authorization_id: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ReportUploadError {
    #[error("failed to run query against database: {0}")]
    QueryFailed(#[from] sqlx::Error),

    #[error("failed to mark the completed upload as current: {0}")]
    MarkCurrentFailed(sqlx::Error),

    #[error("failed to associate finalized uploaded with storage host")]
    NoUploadAssociation(sqlx::Error),

    #[error("failed to register storage grant as redeemed: {0}")]
    RedeemFailed(sqlx::Error),

    #[error("error occurred while recording a blocks present: {0}")]
    UnableToRecordBlock(sqlx::Error),
}

impl IntoResponse for ReportUploadError {
    fn into_response(self) -> Response {
        tracing::error!("{self}");
        let err_msg = serde_json::json!({"msg": "internal server error"});
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
    }
}

async fn associate_upload(
    conn: &mut DatabaseConnection,
    provider_id: &str,
    metadata_id: &str,
    authorization_id: &str,
) -> Result<(), ReportUploadError> {
    sqlx::query!(
        r#"INSERT INTO storage_hosts_metadatas_storage_grants
               (storage_host_id, metadata_id, storage_grant_id)
               VALUES ($1, $2, $3);"#,
        provider_id,
        metadata_id,
        authorization_id,
    )
    .execute(&mut *conn)
    .await
    .map_err(ReportUploadError::NoUploadAssociation)?;

    Ok(())
}

async fn redeem_storage_grant(
    conn: &mut DatabaseConnection,
    provider_id: &str,
    authorization_id: &str,
) -> Result<(), ReportUploadError> {
    sqlx::query!(
        r#"UPDATE storage_grants
               SET redeemed_at = CURRENT_TIMESTAMP
               WHERE storage_host_id = $1
                   AND id = $2
                   AND redeemed_at IS NULL;"#,
        provider_id,
        authorization_id,
    )
    .execute(&mut *conn)
    .await
    .map_err(ReportUploadError::RedeemFailed)?;

    Ok(())
}
