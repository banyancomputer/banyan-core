use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::app::AppState;
use crate::database::models::Bucket;
use crate::extractors::UserIdentity;

pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
    Path(bucket_id): Path<Uuid>,
) -> Result<Response, DeleteBucketError> {
    let bucket_id = bucket_id.to_string();
    let user_id = user_identity.id().to_string();

    let database = state.database();
    let mut conn = database.begin().await?;

    // Check that the bucket exists, is active, and is owned by the user
    if !Bucket::is_owned_by_user_id(&mut conn, &bucket_id, &user_id).await? {
        let err_msg = serde_json::json!({"msg": "not found"});
        return Ok((StatusCode::NOT_FOUND, Json(err_msg)).into_response());
    }

    Bucket::delete(&mut conn, &bucket_id).await?;

    conn.commit().await?;

    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteBucketError {
    #[error("failed to run query: {0}")]
    QueryFailure(#[from] sqlx::Error),
}

impl IntoResponse for DeleteBucketError {
    fn into_response(self) -> Response {
        tracing::error!("internal error handling bucket usage request: {self}");
        let err_msg = serde_json::json!({"msg": "a backend service issue encountered an error"});
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
    }
}
