use axum::extract::{Json, Path};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use uuid::Uuid;

use crate::api::storage::requests;
use crate::db::models;
use crate::extractors::{DbConn, StorageHost};

/// Finalize a metadata upload from a storage host
pub async fn finalize_upload(
    storage_host: StorageHost,
    mut db_conn: DbConn,
    Path(metadata_id): Path<Uuid>,
    Json(finalize_upload_request): Json<requests::FinalizeUpload>,
) -> impl IntoResponse {
    let metadata_id = metadata_id.to_string();
    // Set the real size of the data according to the storage host, and set the metadata state to current
    let data_size = finalize_upload_request.data_size as i64;
    let current = models::MetadataState::Current.to_string();
    let maybe_updated_metadata_bucket_id = sqlx::query_as!(
        models::CreatedResource,
        r#"UPDATE metadata SET state = $1, data_size = $2 WHERE id = $3 RETURNING bucket_id as id;"#,
        current,
        data_size,
        metadata_id,
    )
    .fetch_one(&mut *db_conn.0)
    .await;
    let bucket_id = match maybe_updated_metadata_bucket_id {
        Ok(umb) => umb.id,
        Err(err) => {
            tracing::error!("unable to update bucket metadata after push: {err}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error".to_string(),
            )
                .into_response();
        }
    };

    // Downgrade other metadata for this bucket to outdated if they were in current state
    // Except for the metadata that was just updated
    let outdated = models::MetadataState::Outdated.to_string();
    let maybe_updated_metadata = sqlx::query_as!(
        models::CreatedResource,
        r#"UPDATE metadata SET state = $1 WHERE bucket_id = $2 AND state = $3 AND id != $4 RETURNING id;"#,
        outdated,
        bucket_id,
        current,
        metadata_id,
    ).fetch_optional(&mut *db_conn.0).await;

    match maybe_updated_metadata {
        Ok(_) => {}
        Err(err) => {
            tracing::error!("unable to update bucket metadata after push: {err}");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error".to_string(),
            )
                .into_response();
        }
    }

    // Read over the normalized CIDs (base64 url encoded) and update the blocks table
    let normalized_cids = finalize_upload_request.normalized_cids;
    let mut normalized_cids_iter = normalized_cids.iter();
    while let Some(normalized_cid) = normalized_cids_iter.next() {
        // Try inserting the block into the blocks table if it doesn't exist
        let maybe_block = sqlx::query(
            r#"
                INSERT OR IGNORE INTO
                    blocks (cid)
                    VALUES ($1);
            "#,
        )
        .bind(normalized_cid.clone())
        .execute(&mut *db_conn.0)
        .await;
        match maybe_block {
            Ok(_) => {}
            Err(err) => {
                tracing::error!("unable to insert block in blocks table {err}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                )
                    .into_response();
            }
        }

        let normalized_cid = normalized_cid.to_string();
        let maybe_block_id =
            sqlx::query_scalar!(r#"SELECT id FROM blocks WHERE cid = $1;"#, normalized_cid,)
                .fetch_one(&mut *db_conn.0)
                .await;
        let block_id = match maybe_block_id {
            Ok(block_id) => block_id,
            Err(err) => {
                tracing::error!("unable to get block id from blocks table {err}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                )
                    .into_response();
            }
        };

        let maybe_block_location_id = sqlx::query(
            r#"
                    INSERT INTO
                        block_locations (block_id, metadata_id, storage_host_id)
                        VALUES ($1, $2, $3);
                "#,
        )
        .bind(block_id)
        .bind(metadata_id.clone())
        .bind(storage_host.id().to_string())
        .execute(&mut *db_conn.0)
        .await;
        match maybe_block_location_id {
            Ok(_) => {}
            Err(err) => {
                tracing::error!("unable to insert block location in block_locations table {err}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                )
                    .into_response();
            }
        }
    }
    (StatusCode::NO_CONTENT, ()).into_response()
}
