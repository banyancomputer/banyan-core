use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use uuid::Uuid;

use crate::app::AppState;
use crate::database::models::{BucketAccess, BucketAccessState};
use crate::extractors::UserIdentity;

pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
    Path(bucket_id): Path<Uuid>,
    Json(request): Json<RevokeBucketAccessRequest>,
) -> Result<Response, RevokeBucketAccessError> {
    let _bucket_id = bucket_id.to_string();
    let database = state.database();
    let mut conn = database.acquire().await?;
    let _user_id = user_identity.id().to_string();

    let bucket_access = BucketAccess::by_fingerprint(&mut conn, &request.fingerprint).await?;
    if bucket_access.state != BucketAccessState::Approved {
        return Err(RevokeBucketAccessError::Unauthorized);
    }

    sqlx::query!(
        r#"
            UPDATE bucket_access
            SET state = 'revoked'
            WHERE user_key_id = $1
            AND bucket_id = $2;
        "#,
        bucket_access.user_key_id,
        bucket_access.bucket_id,
    )
    .execute(&database)
    .await?;

    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

#[derive(Debug, Deserialize)]
pub struct RevokeBucketAccessRequest {
    pub fingerprint: String,
}

#[derive(Debug, thiserror::Error)]
pub enum RevokeBucketAccessError {
    #[error("key is unauthorized for API use")]
    Unauthorized,

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for RevokeBucketAccessError {
    fn into_response(self) -> Response {
        match self {
            RevokeBucketAccessError::Unauthorized => {
                let err_msg = serde_json::json!({"msg": "unauthorized"});
                (StatusCode::UNAUTHORIZED, Json(err_msg)).into_response()
            }
            RevokeBucketAccessError::Database(err) => match err {
                sqlx::Error::RowNotFound => {
                    let err_msg = serde_json::json!({"msg": "not found"});
                    (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
                }
                _ => {
                    let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
                }
            },
        }
    }
}
