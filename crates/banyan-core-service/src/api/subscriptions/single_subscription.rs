use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::api::models::ApiSubscription;
use crate::app::AppState;
use crate::database::models::{Subscription, User};
use crate::extractors::UserIdentity;

pub async fn handler(
    user_id: UserIdentity,
    State(state): State<AppState>,
    Path(subscription_id): Path<Uuid>,
) -> Result<Response, SingleSubscriptionError> {
    let database = state.database();
    let mut conn = database.acquire().await?;

    let sub_str_id = subscription_id.to_string();
    let subscription = match Subscription::find_by_id(&mut conn, &sub_str_id).await? {
        Some(sub) => sub,
        None => return Err(SingleSubscriptionError::NotFound),
    };

    let user_id = user_id.id().to_string();
    let current_user = User::by_id(&mut conn, &user_id).await?;
    let active_sub_id = current_user.active_subscription_id();

    // If its not visible and not associated with the current user don't acknowledge its existance
    if !subscription.visible && subscription.id != active_sub_id {
        return Err(SingleSubscriptionError::NotFound);
    }

    let mut api_sub = ApiSubscription::from(subscription);
    api_sub.set_active_if_match(&active_sub_id);

    Ok((StatusCode::OK, Json(api_sub)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum SingleSubscriptionError {
    #[error("database query failed: {0}")]
    DatabaseFailure(#[from] sqlx::Error),

    #[error("subscription not found")]
    NotFound,
}

impl IntoResponse for SingleSubscriptionError {
    fn into_response(self) -> Response {
        match self {
            SingleSubscriptionError::DatabaseFailure(_) => {
                tracing::error!("error from database: {self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            SingleSubscriptionError::NotFound => {
                let err_msg = serde_json::json!({"msg": "not found"});
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            }
        }
    }
}
