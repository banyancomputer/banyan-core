mod checkout_session_events;
mod customer_subscription_events;
mod invoice_events;
mod payment_intent_events;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use stripe::{EventObject, EventType};

use crate::app::AppState;
use crate::extractors::StripeEvent;

pub async fn handler(
    State(state): State<AppState>,
    StripeEvent(event): StripeEvent,
) -> Result<Response, StripeWebhookError> {
    let database = state.database();
    let mut conn = database.begin().await?;

    match (event.type_, &event.data.object) {
        // We don't need to handle, but don't want to log
        (EventType::CustomerCreated, EventObject::Customer(_)) => (),
        (EventType::InvoiceUpcoming, EventObject::Invoice(_)) => (),

        (EventType::InvoiceCreated, EventObject::Invoice(invoice)) => {
            invoice_events::created(&mut conn, invoice).await?
        }
        (EventType::InvoiceFinalizationFailed, EventObject::Invoice(invoice)) => {
            invoice_events::status_update(&mut conn, invoice).await?
        }
        (EventType::InvoiceFinalized, EventObject::Invoice(invoice)) => {
            invoice_events::status_update(&mut conn, invoice).await?
        }
        (EventType::InvoicePaid, EventObject::Invoice(invoice)) => {
            invoice_events::status_update(&mut conn, invoice).await?
        }
        (EventType::InvoicePaymentActionRequired, EventObject::Invoice(invoice)) => {
            invoice_events::status_update(&mut conn, invoice).await?
        }
        (EventType::InvoicePaymentFailed, EventObject::Invoice(invoice)) => {
            invoice_events::status_update(&mut conn, invoice).await?
        }
        // This one should probably be handled as a special case as we can glean some extra data
        // from it, but its not a high priority
        (EventType::InvoiceUpdated, EventObject::Invoice(invoice)) => {
            invoice_events::status_update(&mut conn, invoice).await?
        }

        (EventType::CheckoutSessionCompleted, EventObject::CheckoutSession(sess)) => {
            checkout_session_events::completed(&mut conn, sess).await?
        }
        (EventType::CheckoutSessionExpired, EventObject::CheckoutSession(sess)) => {
            checkout_session_events::expired(&mut conn, sess).await?
        }

        (EventType::CustomerSubscriptionCreated, EventObject::Subscription(subscription)) => {
            customer_subscription_events::manage(&mut conn, subscription).await?
        }
        (EventType::CustomerSubscriptionDeleted, EventObject::Subscription(subscription)) => {
            customer_subscription_events::manage(&mut conn, subscription).await?
        }
        (EventType::CustomerSubscriptionPaused, EventObject::Subscription(subscription)) => {
            customer_subscription_events::manage(&mut conn, subscription).await?
        }
        (EventType::CustomerSubscriptionResumed, EventObject::Subscription(subscription)) => {
            customer_subscription_events::manage(&mut conn, subscription).await?
        }
        (EventType::CustomerSubscriptionUpdated, EventObject::Subscription(subscription)) => {
            customer_subscription_events::manage(&mut conn, subscription).await?
        }

        (EventType::PaymentIntentCreated, EventObject::PaymentIntent(intent)) => {
            payment_intent_events::update_status(&mut conn, intent).await?
        }
        (EventType::PaymentIntentSucceeded, EventObject::PaymentIntent(intent)) => {
            payment_intent_events::update_status(&mut conn, intent).await?
        }

        _ => tracing::warn!("received unknown stripe webhook event: {event:?}"),
    }

    conn.commit().await?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum StripeWebhookError {
    #[error("database query failures: {0}")]
    DatabaseFailure(#[from] sqlx::Error),

    #[error("stripe webhook payload was missing required data")]
    MissingData,

    #[error("unable to locate associated data with webhook")]
    MissingTarget,
}

impl IntoResponse for StripeWebhookError {
    fn into_response(self) -> Response {
        match &self {
            StripeWebhookError::MissingTarget => {
                let err_msg = serde_json::json!({"msg": "not found"});
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            }
            StripeWebhookError::MissingData => {
                let err_msg = serde_json::json!({"msg": "missing expected data"});
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            _ => {
                tracing::error!("a stripe webhook error occurred: {self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
