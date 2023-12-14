use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use uuid::Uuid;

use crate::app::AppState;
use crate::database::models::{MetadataState, PartialMetadataWithSnapshot};
use crate::database::Database;
use crate::extractors::StorageProviderIdentity;

/// When a client finishes uploading their data to either staging or a storage host, the storage
/// host will make a request to this end point letting us know that we have all the data safely
/// stored and can mark the associated metadata as ready to be consumed by downstream clients.
#[axum::debug_handler]
pub async fn handler(
    storage_provider: StorageProviderIdentity,
    State(state): State<AppState>,
    Path(metadata_id): Path<Uuid>,
    Json(request): Json<ReportUploadRequest>,
) -> Result<Response, ReportUploadError> {
    let db_metadata_id = metadata_id.to_string();

    let database = state.database();

    redeem_storage_grant(
        &database,
        &storage_provider.id,
        &request.storage_authorization_id,
    )
    .await?;
    associate_upload(
        &database,
        &storage_provider.id,
        &db_metadata_id,
        &request.storage_authorization_id,
    )
    .await?;

    for block_cid in request.normalized_cids.iter() {
        sqlx::query!("INSERT OR IGNORE INTO blocks (cid) VALUES ($1);", block_cid)
            .execute(&database)
            .await
            .map_err(ReportUploadError::UnableToRecordBlock)?;

        let block_id = sqlx::query_scalar!("SELECT id FROM blocks WHERE cid = $1", block_cid)
            .fetch_one(&database)
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
        .execute(&database)
        .await
        .map_err(ReportUploadError::UnableToRecordBlock)?;
    }

    if can_become_current(&database, &db_metadata_id).await? {
        mark_metadata_current(&database, &db_metadata_id, request.data_size).await?;
        PartialMetadataWithSnapshot::mark_outdated_metadata(&database, &db_metadata_id)
            .await
            .map_err(ReportUploadError::MarkOutdatedFailed)?;
    }

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

impl IntoResponse for ReportUploadError {
    fn into_response(self) -> Response {
        tracing::error!("{self}");
        let err_msg = serde_json::json!({"msg": "internal server error"});
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
    }
}

async fn associate_upload(
    database: &Database,
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
    .execute(database)
    .await
    .map_err(ReportUploadError::NoUploadAssociation)?;

    Ok(())
}

async fn can_become_current(
    database: &Database,
    metadata_id: &str,
) -> Result<bool, ReportUploadError> {
    let checked_created_at = sqlx::query_scalar!(
        r#"SELECT created_at FROM metadata WHERE id = $1;"#,
        metadata_id,
    )
    .fetch_one(database)
    .await
    .map_err(ReportUploadError::MarkCurrentFailed)?;

    let maybe_current_created_at = sqlx::query_scalar!(
        r#"SELECT created_at FROM metadata
               WHERE state = 'current'
               ORDER BY created_at DESC
               LIMIT 1;"#
    )
    .fetch_optional(database)
    .await
    .map_err(ReportUploadError::MarkCurrentFailed)?;

    let current_created_at = match maybe_current_created_at {
        Some(cca) => cca,
        None => return Ok(true),
    };

    Ok(checked_created_at > current_created_at)
}

async fn mark_metadata_current(
    database: &Database,
    metadata_id: &str,
    stored_size: i64,
) -> Result<(), ReportUploadError> {
    let current_state = sqlx::query_scalar!(
        r#"SELECT state as 'state: MetadataState'
               FROM metadata
               WHERE id = $1;"#,
        metadata_id,
    )
    .fetch_one(database)
    .await
    .map_err(ReportUploadError::MarkCurrentFailed)?;

    if current_state == MetadataState::Current {
        return Ok(());
    }

    sqlx::query_scalar!(
        r#"UPDATE metadata SET state = 'current', data_size = $2
               WHERE id = $1;"#,
        metadata_id,
        stored_size,
    )
    .execute(database)
    .await
    .map_err(ReportUploadError::MarkCurrentFailed)?;

    Ok(())
}

async fn redeem_storage_grant(
    database: &Database,
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
    .execute(database)
    .await
    .map_err(ReportUploadError::RedeemFailed)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::models::MetadataState;
    use crate::database::test_helpers;

    #[tokio::test]
    async fn test_marking_metadata() {
        let db = test_helpers::setup_database().await;
        let user_id = test_helpers::sample_user(&db, 1).await;
        let first_bucket_id = test_helpers::create_hot_bucket(&db, &user_id, "apples21")
            .await
            .expect("failed to create new bucket");
        let second_bucket_id = test_helpers::create_hot_bucket(&db, &user_id, "oranges23")
            .await
            .expect("failed to create new bucket");
        let first_metadata_id = test_helpers::pending_metadata(&db, &first_bucket_id, 1).await;
        let second_metadata_id = test_helpers::pending_metadata(&db, &second_bucket_id, 1).await;

        // Mark both of them as current
        mark_metadata_current(&db, &first_metadata_id, 1_200_000)
            .await
            .expect("failed to update metadata state");
        PartialMetadataWithSnapshot::mark_outdated_metadata(&db, &first_metadata_id)
            .await
            .expect("failed to mark outdated");

        mark_metadata_current(&db, &second_metadata_id, 1_200_000)
            .await
            .expect("failed to update metadata state");
        PartialMetadataWithSnapshot::mark_outdated_metadata(&db, &second_metadata_id)
            .await
            .expect("failed to mark outdated");

        test_helpers::assert_metadata_state(&db, &first_metadata_id, MetadataState::Current).await;
        test_helpers::assert_metadata_state(&db, &second_metadata_id, MetadataState::Current).await;
    }

    #[tokio::test]
    async fn test_marking_metadata_when_missing_errors() {
        let db = test_helpers::setup_database().await;

        let result = mark_metadata_current(&db, "not-a-real-id", 1_200_000).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_can_become_current() {
        let db = test_helpers::setup_database().await;
        let bucket_id = test_helpers::sample_bucket(&db).await;

        // an unknown ID is an error
        let result = can_become_current(&db, "missing-id").await;
        assert!(result.is_err());

        let older_pending_metadata_id = test_helpers::pending_metadata(&db, &bucket_id, 1).await;

        // no current metadata present, so we can become current
        assert!(can_become_current(&db, &older_pending_metadata_id)
            .await
            .expect("can check"));

        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        test_helpers::current_metadata(&db, &bucket_id, 2).await;

        // there is a newer version, we can't become current anymore
        assert!(!can_become_current(&db, &older_pending_metadata_id)
            .await
            .expect("can check"));

        // the id is newer than the current one and exists
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        let newer_pending_metadata_id = test_helpers::pending_metadata(&db, &bucket_id, 3).await;
        let result = can_become_current(&db, &newer_pending_metadata_id).await;
        dbg!(&result);
        assert!(result.expect("can check"));
    }
}
