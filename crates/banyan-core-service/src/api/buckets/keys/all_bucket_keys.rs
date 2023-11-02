use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::api::models::ApiBucketKey;
use crate::app::AppState;
use crate::database::models::BucketKey;
use crate::extractors::ApiIdentity;

pub async fn handler(
    api_id: ApiIdentity,
    State(state): State<AppState>,
    Path(bucket_id): Path<Uuid>,
) -> Result<Response, AllBucketKeysError> {
    let database = state.database();
    let bucket_id = bucket_id.to_string();

    let query_result = sqlx::query_as!(
        BucketKey,
        "SELECT bk.* FROM bucket_keys AS bk
             JOIN buckets AS b ON bk.bucket_id = b.id
             WHERE b.user_id = $1 AND bk.bucket_id = $2;",
        api_id.user_id,
        bucket_id,
    )
    .fetch_all(&database)
    .await
    .map_err(AllBucketKeysError::DatabaseFailure)?;

    // note: this also includes user_id which wasn't being returned before and may cause
    // compatibility issues

    let buckets: Vec<_> = query_result.into_iter().map(ApiBucketKey::from).collect();
    Ok((StatusCode::OK, Json(buckets)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum AllBucketKeysError {
    #[error("failed to query the database: {0}")]
    DatabaseFailure(sqlx::Error),
}

impl IntoResponse for AllBucketKeysError {
    fn into_response(self) -> Response {
        tracing::error!("failed to lookup all bucket keys: {self}");
        let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
    }
}
