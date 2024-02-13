use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::api::models::ApiNotification;
use crate::app::AppState;
use crate::database::models::{Notification, NotificationSeverity};
use crate::extractors::UserIdentity;

pub async fn handler(
    user_identity: UserIdentity,
    State(state): State<AppState>,
) -> Result<Response, AllNotificationsError> {
    let database = state.database();

    let user_id = user_identity.id().to_string();
    let query_result = sqlx::query_as!(
        Notification,
        r#"
            SELECT id, user_id, dismissable, message, message_key, severity as 'severity: NotificationSeverity', created_at 
            FROM notifications
            WHERE user_id = $1;
        "#,
        user_id,
    )
    .fetch_all(&database)
    .await
    .map_err(AllNotificationsError::DatabaseFailure)?;

    let notifications: Vec<_> = query_result
        .into_iter()
        .map(ApiNotification::from)
        .collect();
    Ok((StatusCode::OK, Json(notifications)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum AllNotificationsError {
    #[error("failed to query the database: {0}")]
    DatabaseFailure(sqlx::Error),
}

impl IntoResponse for AllNotificationsError {
    fn into_response(self) -> Response {
        tracing::error!("failed to lookup all notifications: {self}");
        let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
        (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
    }
}
