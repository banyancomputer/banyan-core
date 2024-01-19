use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::app::{AppState, StripeHelperError};
use crate::database::models::{Subscription, User};
use crate::extractors::{ServerBase, UserIdentity};

pub async fn handler(
    user_id: UserIdentity,
    ServerBase(host_url): ServerBase,
    State(state): State<AppState>,
    Path(requested_subscription_id): Path<Uuid>,
) -> Result<Response, PurchaseSubscriptionError> {
    // API authenticated users are not allowed to go through the stripe purchase flow, give them a
    // nice error indicating as much
    if let UserIdentity::Api(_) = user_id {
        let err_msg =
            serde_json::json!({"msg": "API authentication is unable to complete payment flow"});
        return Ok((StatusCode::PAYMENT_REQUIRED, Json(err_msg)).into_response());
    }

    let database = state.database();
    let mut conn = database.acquire().await?;

    let requested_subscription_id = requested_subscription_id.to_string();
    let mut requested_subscription = match Subscription::find_by_id(&mut conn, &requested_subscription_id).await? {
        Some(sub) => sub,
        None => return Err(PurchaseSubscriptionError::NotFound),
    };

    let user_id = user_id.id().to_string();
    let mut current_user = User::by_id(&mut *conn, &user_id).await?;

    if current_user.pending_subscription().is_some() {
        // The user has already started changing their subscription to another one, and that hasn't
        // been resolved or expired. Until we've settled the matter we can't start another
        // transition.
        //
        // todo(sstelfox): An improvement here would be to proactively call out to stripe and
        // asking about the status of the pending order, but for now we can simply trigger an error
        // if a user wants to rapidly change between subscriptions.
        return Err(PurchaseSubscriptionError::SubChangeInProgress);
    }

    let current_subscription =
        match Subscription::find_by_id(&mut conn, &current_user.active_subscription_id).await? {
            Some(sub) => sub,
            None => return Err(PurchaseSubscriptionError::NotFound),
        };

    if current_subscription.id == requested_subscription.id {
        let err_msg = serde_json::json!({"msg": "plan is already enabled"});
        return Ok((StatusCode::OK, Json(err_msg)).into_response());
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

    // Note: our subscriptions can be re-used by multiple users, but stripe subscriptions are tied
    // to a specific customer instance and so always need to be generated (we can reused products
    // and prices though).
    let stripe_subscription = stripe_helper
        .realize_subscription(&mut current_user, &mut requested_subscription)
        .await
        .map_err(PurchaseSubscriptionError::StripeSetupError)?;

    let checkout_session = stripe_helper
        .checkout_subscription(&host_url, &current_user, &requested_subscription, &stripe_subscription)
        .await
        .map_err(PurchaseSubscriptionError::StripeSetupError)?;

    let msg = serde_json::json!({"checkout_url": checkout_session.url});
    Ok((StatusCode::OK, Json(msg)).into_response())
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

    #[error("subscription change is already in progress")]
    SubChangeInProgress,
}

impl IntoResponse for PurchaseSubscriptionError {
    fn into_response(self) -> Response {
        match self {
            PurchaseSubscriptionError::NotFound => {
                let err_msg = serde_json::json!({"msg": "not found"});
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            }
            PurchaseSubscriptionError::SubChangeInProgress => {
                let err_msg = serde_json::json!({"msg": "subscription change is already in progress"});
                (StatusCode::CONFLICT, Json(err_msg)).into_response()
            }
            _ => {
                tracing::error!("purchase subscription error: {self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
