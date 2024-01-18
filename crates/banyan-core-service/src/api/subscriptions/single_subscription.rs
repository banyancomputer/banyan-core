use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::api::models::ApiSubscription;
use crate::app::AppState;
use crate::database::models::Subscription;
use crate::extractors::UserIdentity;

pub async fn handler(
    user_id: UserIdentity,
    State(state): State<AppState>,
    Path(subscription_id): Path<Uuid>,
) -> Result<Response, SingleSubscriptionError> {
    let database = state.database();
    let mut conn = database.acquire().await?;

    let user_id = user_id.id().to_string();
    let current_sub_id = sqlx::query_scalar!("SELECT subscription_id as 'subscription_id!' FROM users WHERE id = $1;", user_id)
        .fetch_one(&mut *conn)
        .await?;

    let sub_str_id = subscription_id.to_string();
    let db_sub = match Subscription::find_by_id(&mut conn, &sub_str_id).await? {
        Some(sub) => sub,
        None => return Err(SingleSubscriptionError::NotFound),
    };

    // If its not visible and not associated with the current user don't acknowledge its existance
    if !db_sub.visible && db_sub.id != current_sub_id {
        return Err(SingleSubscriptionError::NotFound);
    }

    let mut api_sub = ApiSubscription::from(db_sub);
    api_sub.set_active_if_match(&current_sub_id);

    Ok((StatusCode::OK, Json(api_sub)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum SingleSubscriptionError {
    #[error("database query failed: {0}")]
    DatabaseFailure(#[from] sqlx::Error),

    #[error("")]
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
