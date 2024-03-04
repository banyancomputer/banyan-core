use axum::extract::{Extension, Path};
use axum::response::{IntoResponse, Response};
use axum::Json;
use http::StatusCode;

use crate::app::AppState;
use crate::database::models::{Subscription, User};
use crate::extractors::AdminIdentity;
use crate::pricing::DEFAULT_SUBSCRIPTION_KEY;

pub async fn handler(
    _: AdminIdentity,
    Path(user_id): Path<String>,
    Extension(state): Extension<AppState>,
) -> Result<Response, ResetUserError> {
    let database = state.database();
    let mut conn = database.acquire().await?;
    let user = User::find_by_id(&mut conn, &user_id)
        .await?.ok_or(ResetUserError::UserNotFound)?;

    let subscription = Subscription::find_by_id(&mut conn, &user.subscription_id)
        .await?
        .ok_or(ResetUserError::SubscriptionNotFound(user.subscription_id))?;

    if subscription.service_key != DEFAULT_SUBSCRIPTION_KEY {
        return Err(ResetUserError::NotOnStarterPlan);
    }

    sqlx::query!(
        r#"DELETE FROM oauth_provider_accounts WHERE user_id = $1;"#,
        user_id
    )
    .execute(&database)
    .await
    .map_err(ResetUserError::DatabaseFailure)?;

    sqlx::query!(
        r#"DELETE FROM device_api_keys WHERE user_id = $1;"#,
        user_id
    )
    .execute(&database)
    .await
    .map_err(ResetUserError::DatabaseFailure)?;

    Ok((StatusCode::OK, Json("User reset successfully")).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum ResetUserError {
    #[error("failed to query the database: {0}")]
    DatabaseFailure(#[from] sqlx::Error),
    #[error("subscription not found: {0}")]
    SubscriptionNotFound(String),
    #[error("user not found")]
    UserNotFound,
    #[error("user is not on the starter plan")]
    NotOnStarterPlan,
}

impl IntoResponse for ResetUserError {
    fn into_response(self) -> Response {
        match &self {
            ResetUserError::DatabaseFailure(_) => {
                tracing::error!("error from database: {self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
            ResetUserError::UserNotFound | ResetUserError::NotOnStarterPlan | ResetUserError::SubscriptionNotFound(_) => {
                let err_msg = serde_json::json!({ "msg": self.to_string() });
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
        }
    }
}
