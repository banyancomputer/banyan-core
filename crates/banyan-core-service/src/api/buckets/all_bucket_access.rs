use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::api::models::ApiBucketAccess;
use crate::app::AppState;
use crate::extractors::UserIdentity;

pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
    Path(bucket_id): Path<Uuid>,
) -> Result<Response, AllBucketAccessError> {
    let database = state.database();
    let bucket_id = bucket_id.to_string();
    let user_id = user_identity.id().to_string();
    let access = sqlx::query_as!(
        ApiBucketAccess,
        r#"
            SELECT ba.user_key_id, ba.bucket_id, uk.fingerprint, ba.approved
            FROM bucket_access AS ba
            JOIN user_keys AS uk ON ba.user_key_id = uk.id
            WHERE uk.user_id = $1
            AND ba.bucket_id = $2;
        "#,
        user_id,
        bucket_id,
    )
    .fetch_all(&database)
    .await
    .map_err(AllBucketAccessError::DatabaseFailure)?;

    Ok((StatusCode::OK, Json(access)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum AllBucketAccessError {
    #[error("failed to query the database: {0}")]
    DatabaseFailure(sqlx::Error),
}

impl IntoResponse for AllBucketAccessError {
    fn into_response(self) -> Response {
        tracing::error!("failed to lookup all bucket keys: {self}");
        let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
    }
}