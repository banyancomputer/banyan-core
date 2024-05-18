use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use uuid::Uuid;

use crate::app::AppState;
use crate::extractors::UserIdentity;

/// Register a new device api key with an account
pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
    Path(key_id): Path<Uuid>,
    Json(request): Json<RenameUserKeyRequest>,
) -> Result<Response, RenameUserKeyError> {
    let database = state.database();
    let key_id = key_id.to_string();
    let user_id = user_identity.id().to_string();
    let result = sqlx::query!(
        r#"
            UPDATE user_keys 
            SET name = $1
            WHERE id = $2
            AND user_id = $3;
        "#,
        request.name,
        key_id,
        user_id,
    )
    .execute(&database)
    .await
    .map_err(RenameUserKeyError::FailedToRenameKey)?;

    if result.rows_affected() == 0 {
        Err(RenameUserKeyError::NotFound)
    } else {
        Ok((StatusCode::NO_CONTENT, ()).into_response())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RenameUserKeyError {
    #[error("failed to store device API key: {0}")]
    FailedToRenameKey(sqlx::Error),

    #[error("no matching key")]
    NotFound,
}

impl IntoResponse for RenameUserKeyError {
    fn into_response(self) -> Response {
        match self {
            RenameUserKeyError::FailedToRenameKey(_) => {
                tracing::error!("failed to rename user key: {self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            RenameUserKeyError::NotFound => {
                let err_msg = serde_json::json!({"msg": "not found"});
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            }
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct RenameUserKeyRequest {
    name: String,
}
