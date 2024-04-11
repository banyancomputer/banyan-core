use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::app::AppState;
use crate::database::models::{Bucket, ExplicitBigInt};
use crate::extractors::UserIdentity;

pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
    Path(bucket_id): Path<Uuid>,
) -> Result<Response, BucketUsageError> {
    let bucket_id = bucket_id.to_string();
    let user_id = user_identity.id().to_string();
    let database = state.database();
    let mut conn = database.acquire().await?;

    // Note: this also enforeces that the bucket does not have `deleted_at` set
    if !Bucket::is_owned_by_user_id(&mut conn, &bucket_id, &user_id).await? {
        let err_msg = serde_json::json!({"msg": "not found"});
        return Ok((StatusCode::NOT_FOUND, Json(err_msg)).into_response());
    }

    let user_id = user_identity.id().to_string();
    let query_result = sqlx::query_as!(
        ExplicitBigInt,
        r#"SELECT COALESCE(SUM(m.data_size), 0) as big_int
               FROM metadata m
               JOIN buckets b ON m.bucket_id = b.id
               WHERE b.user_id = $1 AND b.id = $2;"#,
        user_id,
        bucket_id,
    )
    .fetch_one(&mut *conn)
    .await;

    match query_result {
        Ok(size) => {
            let resp = serde_json::json!({ "size": size.big_int });
            Ok((StatusCode::OK, Json(resp)).into_response())
        }
        Err(_) => {
            let err_msg = serde_json::json!({ "msg": "bucket not found" });
            Ok((StatusCode::NOT_FOUND, Json(err_msg)).into_response())
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BucketUsageError {
    #[error("failed to run query: {0}")]
    QueryFailure(#[from] sqlx::Error),
}

impl IntoResponse for BucketUsageError {
    fn into_response(self) -> Response {
        tracing::error!("internal error handling bucket usage request: {self}");
        let err_msg = serde_json::json!({"msg": "a backend service issue encountered an error"});
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
    }
}
