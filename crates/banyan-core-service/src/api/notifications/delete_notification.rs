use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::app::AppState;
use crate::database::models::Notification;
use crate::extractors::UserIdentity;

pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
    Path(notification_id): Path<Uuid>,
) -> Result<Response, DeleteNotificationError> {
    let notification_id = notification_id.to_string();
    let user_id = user_identity.id().to_string();

    let database = state.database();
    let mut conn = database.begin().await?;

    /*
    if !Notification::is_owned_by_user_id(&mut conn, &notification_id, &user_id).await? {
        let err_msg = serde_json::json!({"msg": "not found"});
        return Ok((StatusCode::NOT_FOUND, Json(err_msg)).into_response());
    }

    Notification::delete(&mut conn, &notification_id).await?;
    */

    conn.commit().await?;

    Ok((StatusCode::NO_CONTENT, ()).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteNotificationError {
    #[error("failed to run query: {0}")]
    QueryFailure(#[from] sqlx::Error),
}

impl IntoResponse for DeleteNotificationError {
    fn into_response(self) -> Response {
        tracing::error!("failed to delete notification: {self}");
        let err_msg = serde_json::json!({"msg": "a backend service issue encountered an error"});
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
    }
}
