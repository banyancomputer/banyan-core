use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::api::models::ApiMetadata;
use crate::app::AppState;
use crate::database::models::{Bucket, PartialMetadataWithSnapshot};
use crate::extractors::UserIdentity;

pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
    Path(bucket_id): Path<Uuid>,
) -> Result<Response, CurrentMetadataError> {
    let user_id = user_identity.id().to_string();
    let bucket_id = bucket_id.to_string();

    let mut conn = state.database().acquire().await?;

    if !Bucket::is_owned_by_user_id(&mut conn, &bucket_id, &user_id).await? {
        tracing::info!("bucket wasn't 'owned'");
        let err_msg = serde_json::json!({"msg": "not found"});
        return Ok((StatusCode::NOT_FOUND, Json(err_msg)).into_response());
    }

    let current = PartialMetadataWithSnapshot::locate_current(&mut conn, &bucket_id).await?;

    match current {
        Some(m) => Ok((StatusCode::OK, Json(ApiMetadata::from(m))).into_response()),
        None => {
            let err_msg = serde_json::json!({"msg": "not found"});
            Ok((StatusCode::NOT_FOUND, Json(err_msg)).into_response())
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CurrentMetadataError {
    #[error("a database query failed: {0}")]
    QueryFailed(#[from] sqlx::Error),
}

impl IntoResponse for CurrentMetadataError {
    fn into_response(self) -> Response {
        tracing::error!("failed to retrieve current metadata: {self}");
        let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
    }
}
