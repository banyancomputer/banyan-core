use axum::extract::{Json, Path};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use uuid::Uuid;

use crate::api::storage::requests;
use crate::database::Database;
use crate::db::models;
use crate::extractors::StorageHostToken;

/// Finalize a metadata upload from a storage host
pub async fn finalize_upload(
    _storage_host_token: StorageHostToken,
    database: Database,
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
    .fetch_one(&database)
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
    ).fetch_optional(&database).await;

    match maybe_updated_metadata {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(err) => {
            tracing::error!("unable to update bucket metadata after push: {err}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error".to_string(),
            )
                .into_response()
        }
    }
}
