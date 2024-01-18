use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::app::{AppState, StripeHelper, StripeHelperError};
use crate::database::models::Subscription;
use crate::extractors::UserIdentity;

pub async fn handler(
    user_id: UserIdentity,
    State(state): State<AppState>,
    Path(subscription_id): Path<Uuid>,
) -> Result<Response, PurchaseSubscriptionError> {
    // API authenticated users are not allowed to go through the stripe purchase flow, give them a
    // nice error indicating as much
    if let UserIdentity::Api(_) = user_id {
        let err_msg = serde_json::json!({"msg": "API authentication is unable to complete payment flow"});
        return Ok((StatusCode::PAYMENT_REQUIRED, Json(err_msg)).into_response());
    }

    let database = state.database();
    let mut conn = database.acquire().await?;

    let user_id = user_id.id().to_string();
    let current_sub_id = sqlx::query_scalar!("SELECT subscription_id as 'subscription_id!' FROM users WHERE id = $1;", user_id)
        .fetch_one(&mut *conn)
        .await?;

    let current_sub_id_str = current_sub_id.to_string();
    let current_subscription = match Subscription::find_by_id(&mut conn, &current_sub_id_str).await? {
        Some(sub) => sub,
        None => return Err(PurchaseSubscriptionError::NotFound),
    };

    let subscription_id = subscription_id.to_string();
    let selected_subscription = match Subscription::find_by_id(&mut conn, &subscription_id).await? {
        Some(sub) => sub,
        None => return Err(PurchaseSubscriptionError::NotFound),
    };

    if current_subscription.id == selected_subscription.id {
        let err_msg = serde_json::json!({"msg": "plan is already enabled"});
        return Ok((StatusCode::OK, Json(err_msg)).into_response());
    }

    // For now we don't allow transitioning between subscriptions directly, so a user must be
    // coming from a new plan.
    if current_subscription.service_key != "starter" {
        let err_msg = serde_json::json!({"msg": "can only select plan from the starter (temporary limitation)"});
        return Ok((StatusCode::BAD_REQUEST, Json(err_msg)).into_response());
    }

    let stripe_helper = match state.stripe_helper() {
        Some(sh) => sh,
        None => {
            // The system doesn't have credentials to make stripe based calls, for now I'm going to
            // make this a server error, but I may make this a "debug mode" to just switch the
            // subscription for ease of development... We'll see...
            tracing::warn!("unable to make stripe calls due to missing key");
            return Err(PurchaseSubscriptionError::NoStripeHelper);
        }
    };

    let stripe_checkout_object = stripe_helper
        .realize_subscription(&user_id, &selected_subscription)
        .await
        .map_err(PurchaseSubscriptionError::StripeSetupError)?;

    todo!()
}

#[derive(Debug, thiserror::Error)]
pub enum PurchaseSubscriptionError {
    #[error("database query failed: {0}")]
    DatabaseFailure(#[from] sqlx::Error),

    #[error("unable to purchase subscription without a stripe helper")]
    NoStripeHelper,

    #[error("subscription not found")]
    NotFound,

    #[error("failure occurred setting up stripe to purchase subscription: {0}")]
    StripeSetupError(StripeHelperError),
}

impl IntoResponse for PurchaseSubscriptionError {
    fn into_response(self) -> Response {
        match self {
            PurchaseSubscriptionError::NotFound => {
                let err_msg = serde_json::json!({"msg": "not found"});
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            }
            _ => {
                tracing::error!("purchase subscription error: {self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
