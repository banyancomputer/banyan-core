use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use uuid::Uuid;

use crate::app::AppState;
use crate::database::models::UserKey;
use crate::extractors::UserIdentity;

/// Register a new device api key with an account
pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
    Path(key_id): Path<Uuid>,
    Json(request): Json<RenameUserKeyRequest>,
) -> Result<Response, RenameUserKeyError> {
    let database = state.database();
    let user_id = user_identity.id().to_string();
    let new_key = sqlx::query_as!(
        UserKey,
        r#"
            UPDATE user_keys 
            SET name = $1
            WHERE id = $2
            AND user_id = $3
            RETURNING *;
        "#,
        request.name,
        key_id,
        user_id,
    )
    .fetch_one(&database)
    .await
    .map_err(RenameUserKeyError::FailedToRenameKey)?;

    let resp_msg = serde_json::json!({"key": new_key});
    Ok((StatusCode::OK, Json(resp_msg)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum RenameUserKeyError {
    #[error("failed to store device API key: {0}")]
    FailedToRenameKey(sqlx::Error),
}

impl IntoResponse for RenameUserKeyError {
    fn into_response(self) -> Response {
        match &self {
            _ => {
                tracing::error!("failed to create device api key: {self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}

#[derive(Deserialize)]
pub struct RenameUserKeyRequest {
    name: String,
}
