use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::api::models::ApiSubscription;
use crate::app::AppState;
use crate::database::models::{Subscription, TaxClass, User};
use crate::extractors::UserIdentity;

pub async fn handler(
    user: Option<UserIdentity>,
    State(state): State<AppState>,
) -> Result<Response, AllSubscriptionsError> {
    let database = state.database();
    let mut conn = database.acquire().await?;

    let current_user = match user {
        Some(u) => {
            let user_id = u.id().to_string();
            let current_user = User::by_id(&mut conn, &user_id).await?;
            Some(current_user)
        }
        None => None,
    };

    let mut subscriptions = Subscription::all_public_or_current(
        &mut conn,
        current_user.as_ref().map(|u| u.subscription_id.as_str()),
    )
    .await?;

    if let Some(user) = &current_user {
        // If we know the tax class the subscription will be part of from the user's configuration,
        // we can pre-filter the subscriptions to match
        match user.account_tax_class {
            TaxClass::Business | TaxClass::Personal => {
                subscriptions.retain(|s| s.tax_class == user.account_tax_class);
            }
            _ => (),
        }
    }

    let mut api_subscriptions: Vec<_> = subscriptions
        .into_iter()
        .map(ApiSubscription::from)
        .collect();

    if let Some(user) = current_user {
        for sub in api_subscriptions.iter_mut() {
            sub.set_active_if_match(&user.subscription_id);
        }
    }

    Ok((StatusCode::OK, Json(api_subscriptions)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum AllSubscriptionsError {
    #[error("database query failed: {0}")]
    DatabaseFailure(#[from] sqlx::Error),
}

impl IntoResponse for AllSubscriptionsError {
    fn into_response(self) -> Response {
        match self {
            AllSubscriptionsError::DatabaseFailure(_) => {
                tracing::error!("error from database: {self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
