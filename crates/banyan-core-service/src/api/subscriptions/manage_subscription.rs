use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::app::{AppState, StripeHelperError};
use crate::database::models::User;
use crate::extractors::{ServerBase, UserIdentity};

pub async fn handler(
    user_id: UserIdentity,
    ServerBase(host_url): ServerBase,
    State(state): State<AppState>,
) -> Result<Response, ManageSubscriptionError> {
    let database = state.database();
    let mut conn = database.acquire().await?;

    let stripe_helper = match state.stripe_helper() {
        Some(sh) => sh,
        None => return Err(ManageSubscriptionError::NoStripeHelper),
    };

    let user_id = user_id.id().to_string();
    let current_user = User::by_id(&mut conn, &user_id).await?;

    let stripe_customer_id = match &current_user.stripe_customer_id {
        Some(sci) => sci,
        None => return Err(ManageSubscriptionError::NotFound),
    };

    let billing_session = stripe_helper.portal_session(&host_url, &stripe_customer_id).await?;

    let msg = serde_json::json!({"portal_url": billing_session.url});
    Ok((StatusCode::OK, Json(msg)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum ManageSubscriptionError {
    #[error("database query failed: {0}")]
    DatabaseFailure(#[from] sqlx::Error),

    #[error("stripe isn't configured")]
    NoStripeHelper,

    #[error("unable to manage account of user without stripe association")]
    NotFound,

    #[error("error interacting with stripe: {0}")]
    StripeHelperError(#[from] StripeHelperError),
}

impl IntoResponse for ManageSubscriptionError {
    fn into_response(self) -> Response {
        match self {
            ManageSubscriptionError::NotFound => {
                let err_msg = serde_json::json!({"msg": "account isn't associated with a payment plan"});
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            }
            _ => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
