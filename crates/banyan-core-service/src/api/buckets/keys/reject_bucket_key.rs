use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use super::single_bucket_key::SingleBucketKeyError;
use crate::app::AppState;
use crate::database::models::BucketKey;
use crate::extractors::ApiIdentity;

pub async fn handler(
    api_id: ApiIdentity,
    State(state): State<AppState>,
    Path((bucket_id, bucket_key_id)): Path<(Uuid, Uuid)>,
) -> Result<Response, SingleBucketKeyError> {
    let bucket_id = bucket_id.to_string();
    let bucket_key_id = bucket_key_id.to_string();
    let database = state.database();

    let maybe_bucket_key = sqlx::query_as!(
        BucketKey,
        r#"SELECT bk.* FROM bucket_keys AS bk
               JOIN buckets b ON bk.bucket_id = b.id
               WHERE b.account_id = $1
                   AND bk.bucket_id = $2
                   AND bk.id = $3;"#,
        api_id.account_id,
        bucket_id,
        bucket_key_id,
    )
    .fetch_optional(&database)
    .await
    .map_err(SingleBucketKeyError::DatabaseFailure)?;

    // If the bucket key already exists and is already approved, disqualifying it from rejection
    if let Some(bucket_key) = maybe_bucket_key {
        if bucket_key.approved {
            let err_msg = serde_json::json!({"msg": "bucket key is already approved"});
            return Ok((StatusCode::EXPECTATION_FAILED, Json(err_msg)).into_response());
        }
    }

    let query_result = sqlx::query!(
        r#"DELETE FROM bucket_keys
                WHERE id IN (
                    SELECT bk.id FROM bucket_keys AS bk
                        JOIN buckets AS b ON bk.bucket_id = b.id
                        WHERE b.account_id = $1 AND bk.id = $2 AND bk.bucket_id = $3
                );"#,
        api_id.account_id,
        bucket_key_id,
        bucket_id,
    )
    .execute(&database)
    .await;

    match query_result {
        Ok(_) => Ok((StatusCode::NO_CONTENT, ()).into_response()),
        Err(sqlx::Error::RowNotFound) => {
            let err_msg = serde_json::json!({"msg": "not found"});
            Ok((StatusCode::NOT_FOUND, Json(err_msg)).into_response())
        }
        Err(err) => {
            tracing::error!("failed to delete bucket key: {err}");
            let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
            Ok((StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response())
        }
    }
}
