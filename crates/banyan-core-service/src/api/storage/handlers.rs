use axum::extract::{Json, Path};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use uuid::Uuid;

use crate::api::storage::requests;
use crate::db::models;
use crate::extractors::{DbConn, StorageHostToken};

/// Finalize a metadata upload from a storage host
pub async fn finalize_upload(
    _storage_host_token: StorageHostToken,
    mut db_conn: DbConn,
    Path(metadata_id): Path<Uuid>,
    Json(finalize_upload_request): Json<requests::FinalizeUpload>,
) -> impl IntoResponse {
    let metadata_id = metadata_id.to_string();
    // Set the real size of the data according to the storage host, and set the metadata state to current
    let data_size = finalize_upload_request.data_size as i64;
    let metadata_state = models::MetadataState::Current.to_string();
    let maybe_updated_metadata = sqlx::query_as!(
        models::CreatedResource,
        r#"UPDATE metadata SET state = $1, data_size = $2 WHERE id = $3 RETURNING id;"#,
        metadata_state,
        data_size,
        metadata_id,
    )
    .fetch_one(&mut *db_conn.0)
    .await;
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
