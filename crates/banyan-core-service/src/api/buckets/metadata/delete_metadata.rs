use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::app::AppState;
use crate::database::models::MetadataState;
use crate::extractors::ApiIdentity;

pub async fn handler(
    api_id: ApiIdentity,
    State(state): State<AppState>,
    Path((bucket_id, metadata_id)): Path<(Uuid, Uuid)>,
) -> Result<Response, DeleteMetadataError> {
    let bucket_id = bucket_id.to_string();
    let metadata_id = metadata_id.to_string();

    let database = state.database();
    let mut transaction = database.begin().await.map_err(DeleteMetadataError::TransactionUnavailable)?;

    let metadata_state = sqlx::query_scalar!(
        r#"SELECT m.state as 'state: MetadataState' FROM metadata AS m
               JOIN buckets AS b ON m.bucket_id = b.id
               WHERE b.account_id = $1 AND b.id = $2 AND m.id = $3;"#,
        api_id.account_id,
        bucket_id,
        metadata_id,
    )
    .fetch_optional(&mut *transaction)
    .await
    .map_err(DeleteMetadataError::LookupUnavailable)?
    .ok_or(DeleteMetadataError::NotFound)?;

    if metadata_state == MetadataState::Current {
        // maybe we should explicitly track the previous version instead of guessing... This is
        // best effort as well, if there isn't another metadata entry available it's not an error
        // but a DB query in general is.
        let maybe_previous_metadata_id = sqlx::query_scalar!(
            r#"SELECT m.id FROM metadata AS m
                   WHERE m.bucket_id = $1 AND m.state = 'outdated'
                   ORDER BY updated_at DESC
                   LIMIT 1;"#,
            bucket_id,
        )
        .fetch_optional(&mut *transaction)
        .await
        .map_err(DeleteMetadataError::LookupUnavailable)?;

        if let Some(prev_id) = maybe_previous_metadata_id {
            sqlx::query!(
                r#"UPDATE metadata
                       SET state = 'current',
                           updated_at = CURRENT_TIMESTAMP
                       WHERE id = $1;"#,
                prev_id,
            )
                .execute(&mut *transaction)
                .await
                .map_err(DeleteMetadataError::ReversionFailed)?;
        }
    }

    sqlx::query!(
        r#"UPDATE metadata
               SET state = 'deleted',
                   updated_at = CURRENT_TIMESTAMP
               WHERE id = $1;"#,
        metadata_id,
    )
    .execute(&mut *transaction)
    .await
    .map_err(DeleteMetadataError::FailedDelete)?;

    // todo: need to delete all the hot data stored at various storage hosts

    transaction.commit().await.map_err(DeleteMetadataError::CommitFailed)?;

    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteMetadataError {
    #[error("failed to commit the changes to the database: {0}")]
    CommitFailed(sqlx::Error),

    #[error("failed to delete metadata entry: {0}")]
    FailedDelete(sqlx::Error),

    #[error("failed to query the database for metadata entry: {0}")]
    LookupUnavailable(sqlx::Error),

    #[error("metadata entry wasn't found")]
    NotFound,

    #[error("reverting the metadata version failed")]
    ReversionFailed(sqlx::Error),

    #[error("unable to start a transaction: {0}")]
    TransactionUnavailable(sqlx::Error),
}

impl IntoResponse for DeleteMetadataError {
    fn into_response(self) -> Response {
        match &self {
            DeleteMetadataError::NotFound => {
                let err_msg = serde_json::json!({"msg": "not found"});
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            }
            _ => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
