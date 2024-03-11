use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

use crate::app::AppState;
use crate::database::models::{Subscription, User};
use crate::extractors::UserIdentity;
use crate::GIBIBYTE;

pub async fn handler(
    user_id: UserIdentity,
    State(state): State<AppState>,
) -> Result<Response, UsageLimitError> {
    let database = state.database();
    let mut conn = database.acquire().await?;

    let user_id = user_id.id().to_string();
    let user = User::find_by_id(&mut conn, &user_id)
        .await?
        .ok_or(UsageLimitError::NotFound)?;

    let subscription = Subscription::find_by_id(&mut conn, &user.subscription_id)
        .await?
        .ok_or(UsageLimitError::NotFound)?;

    let resp = UsageLimitResponse {
        soft_hot_storage_limit: subscription.included_hot_storage * GIBIBYTE,
        hard_hot_storage_limit: subscription.hot_storage_hard_limit.map(|l| l * GIBIBYTE),
        soft_archival_storage_limit: subscription.included_archival * GIBIBYTE,
        hard_archival_storage_limit: subscription.archival_hard_limit.map(|l| l * GIBIBYTE),
        size: subscription.included_hot_storage * GIBIBYTE,
    };

    Ok((StatusCode::OK, Json(resp)).into_response())
}

#[derive(Serialize)]
struct UsageLimitResponse {
    soft_hot_storage_limit: i64,

    #[serde(skip_serializing_if = "Option::is_none")]
    hard_hot_storage_limit: Option<i64>,

    soft_archival_storage_limit: i64,

    #[serde(skip_serializing_if = "Option::is_none")]
    hard_archival_storage_limit: Option<i64>,

    // legacy option, should be removed as soon as the frontend doesn't use this
    size: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum UsageLimitError {
    #[error("a database failure occurred: {0}")]
    DatabaseFailure(#[from] sqlx::Error),

    #[error("associated data couldn't be found")]
    NotFound,
}

impl IntoResponse for UsageLimitError {
    fn into_response(self) -> Response {
        match self {
            UsageLimitError::NotFound => {
                let err_msg = serde_json::json!({"msg": "not found"});
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            }
            _ => {
                tracing::error!("usage lookup error: {self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
