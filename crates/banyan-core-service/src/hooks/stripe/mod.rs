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

pub async fn handler(State(state): State<AppState>, StripeEvent(event): StripeEvent) -> Result<Response, StripeWebhookError> {
    let database = state.database();
    let mut conn = database.acquire().await?;

    match (event.type_, &event.data.object) {
        // We don't need to handle, but don't want to log
        (EventType::CustomerCreated, EventObject::Customer(_)) => (),
        (EventType::InvoiceUpcoming, EventObject::Invoice(_)) => (),

        (EventType::InvoiceCreated, EventObject::Invoice(invoice)) => invoice_events::created(&mut *conn, invoice).await?,
        (EventType::InvoiceFinalizationFailed, EventObject::Invoice(invoice)) => invoice_events::finalization_failed(&mut *conn, invoice).await?,
        (EventType::InvoiceFinalized, EventObject::Invoice(invoice)) => invoice_events::finalized(&mut *conn, invoice).await?,
        (EventType::InvoicePaid, EventObject::Invoice(invoice)) => invoice_events::paid(&mut *conn, invoice).await?,
        (EventType::InvoicePaymentActionRequired, EventObject::Invoice(invoice)) => invoice_events::payment_action_required(&mut *conn, invoice).await?,
        (EventType::InvoicePaymentFailed, EventObject::Invoice(invoice)) => invoice_events::payment_failed(&mut *conn, invoice).await?,
        (EventType::InvoiceUpdated, EventObject::Invoice(invoice)) => invoice_events::updated(&mut *conn, invoice).await?,

        (EventType::CheckoutSessionCompleted, EventObject::CheckoutSession(sess)) => checkout_session_events::completed(&mut *conn, sess).await?,
        (EventType::CheckoutSessionExpired, EventObject::CheckoutSession(sess)) => checkout_session_events::expired(&mut *conn, sess).await?,

        (EventType::CustomerSubscriptionCreated, EventObject::Subscription(subscription)) => customer_subscription_events::created(&mut *conn, subscription).await?,
        (EventType::CustomerSubscriptionDeleted, EventObject::Subscription(subscription)) => customer_subscription_events::deleted(&mut *conn, subscription).await?,
        (EventType::CustomerSubscriptionPaused, EventObject::Subscription(subscription)) => customer_subscription_events::paused(&mut *conn, subscription).await?,
        (EventType::CustomerSubscriptionResumed, EventObject::Subscription(subscription)) => customer_subscription_events::resumed(&mut *conn, subscription).await?,
        (EventType::CustomerSubscriptionUpdated, EventObject::Subscription(subscription)) => customer_subscription_events::updated(&mut *conn, subscription).await?,

        (EventType::PaymentIntentCreated, EventObject::PaymentIntent(intent)) => payment_intent_events::created(&mut *conn, intent).await?,
        (EventType::PaymentIntentSucceeded, EventObject::PaymentIntent(intent)) => payment_intent_events::succeeded(&mut *conn, intent).await?,

        _ => tracing::warn!("received unknown stripe webhook event: {event:?}"),
    }

    let msg = serde_json::json!({"msg": "ok"});
    Ok((StatusCode::OK, Json(msg)).into_response())
}

#[derive(Debug, thiserror::Error)]
pub enum StripeWebhookError {
    #[error("database query failures: {0}")]
    DatabaseFailure(#[from] sqlx::Error),
}

impl IntoResponse for StripeWebhookError {
    fn into_response(self) -> Response {
        match &self {
            StripeWebhookError::DatabaseFailure(_) => {
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
