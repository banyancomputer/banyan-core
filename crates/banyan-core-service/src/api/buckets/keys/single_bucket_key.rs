use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use uuid::Uuid;

use crate::app::AppState;
use crate::database::models::BucketKey;
use crate::extractors::ApiToken;
use crate::api::models::ApiBucketKey;

pub async fn handler(
    api_token: ApiToken,
    State(state): State<AppState>,
    Path((bucket_id, bucket_key_id)): Path<(Uuid, Uuid)>,
) -> Result<Response, SingleBucketKeyError> {
    let database = state.database();

    let account_id = api_token.subject();
    let bucket_id = bucket_id.to_string();
    let bucket_key_id = bucket_key_id.to_string();

    let maybe_bucket_key = sqlx::query_as!(
        BucketKey,
        r#"SELECT bk.* FROM bucket_keys AS bk
               JOIN buckets b ON bk.bucket_id = b.id
               WHERE b.account_id = $1 AND bk.bucket_id = $2;"#,
        account_id,
        bucket_id,
    )
    .fetch_optional(&database)
    .await
    .map_err(SingleBucketKeyError::DatabaseFailure)?;

    if let Some(bk) = maybe_bucket_key {
        Ok((StatusCode::OK, Json(ApiBucketKey::from(bk))).into_response())
    } else {
        let err_msg = serde_json::json!({"msg": "not found"});
        Ok((StatusCode::NOT_FOUND, Json(err_msg)).into_response())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SingleBucketKeyError {
    #[error("failed to query the database: {0}")]
    DatabaseFailure(sqlx::Error),
}

impl IntoResponse for SingleBucketKeyError {
    fn into_response(self) -> Response {
        tracing::error!("failed to lookup bucket key: {self}");
        let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
    }
}
