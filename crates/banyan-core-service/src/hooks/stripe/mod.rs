mod invoice_events;
mod session_events;
mod subscription_events;

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
    use {EventObject as EO, EventType as ET};

    let database = state.database();
    let mut conn = database.begin().await?;

    match (event.type_, &event.data.object) {
        // We don't track customer data state
        (ET::CustomerCreated, EO::Customer(_)) => (),
        (ET::CustomerUpdated, EO::Customer(_)) => (),
        (ET::PaymentMethodAttached, EO::PaymentMethod(_)) => (),

        // We track the status of the subscription invoices not individual payments
        (ET::ChargeSucceeded, EO::Charge(_)) => (),
        (ET::PaymentIntentSucceeded, EO::PaymentIntent(_)) => (),
        (ET::PaymentIntentCreated, EO::PaymentIntent(_)) => (),

        // These are dynamic managed objects by our server, we don't need to know when we change
        // them
        (ET::PlanCreated, EO::Plan(_)) => (),
        (ET::PriceCreated, EO::Price(_)) => (),

        // We may not need to handle these as our invoice status is much more accurate about
        // _which_ subscription we should consider active if there are multiple (such as
        // transitioning between subscriptions).
        (ET::CustomerSubscriptionCreated, EO::Subscription(sub)) => subscription_events::handler(&mut conn, sub).await?,
        (ET::CustomerSubscriptionUpdated, EO::Subscription(sub)) => subscription_events::handler(&mut conn, sub).await?,

        // We don't support pausing and resuming our subscription, and the deletion/cancel workflow
        // is handled by our invoice synchronization
        (ET::CustomerSubscriptionDeleted, EO::Subscription(_)) => (),
        (ET::CustomerSubscriptionPaused, EO::Subscription(_)) => (),
        (ET::CustomerSubscriptionResumed, EO::Subscription(_)) => (),

        (ET::InvoiceCreated, EO::Invoice(inv)) => invoice_events::creation_handler(&mut conn, inv).await?,
        (ET::InvoiceUpcoming, EO::Invoice(_)) => (),

        (ET::InvoiceFinalizationFailed, EO::Invoice(inv)) => invoice_events::update_handler(&mut conn, inv).await?,
        (ET::InvoiceFinalized, EO::Invoice(inv)) => invoice_events::update_handler(&mut conn, inv).await?,
        (ET::InvoicePaid, EO::Invoice(inv)) => invoice_events::update_handler(&mut conn, inv).await?,
        (ET::InvoicePaymentActionRequired, EO::Invoice(inv)) => invoice_events::update_handler(&mut conn, inv).await?,
        (ET::InvoicePaymentFailed, EO::Invoice(inv)) =>  invoice_events::update_handler(&mut conn, inv).await?,
        (ET::InvoicePaymentSucceeded, EO::Invoice(inv)) => invoice_events::update_handler(&mut conn, inv).await?,
        (ET::InvoiceUpdated, EO::Invoice(inv)) => invoice_events::update_handler(&mut conn, inv).await?,

        // This should be the only place where a subscription actually changes other than being
        // canceled / run out of paid time. This event indicates the customer has finished (and
        // payment has been confirmed) for a particular subscription.
        (ET::CheckoutSessionCompleted, EO::CheckoutSession(sess)) => session_events::handler(&mut conn, sess).await?,

        (ET::CheckoutSessionExpired, EO::CheckoutSession(_)) => (),

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
            StripeWebhookError::MissingData => {
                let err_msg = serde_json::json!({"msg": "missing expected data"});
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            StripeWebhookError::MissingTarget => {
                let err_msg = serde_json::json!({"msg": "not found"});
                (StatusCode::NOT_FOUND, Json(err_msg)).into_response()
            }
            _ => {
                tracing::error!("a stripe webhook error occurred: {self}");
                let err_msg = serde_json::json!({"msg": "backend service experienced an issue servicing the request"});
                (StatusCode::INTERNAL_SERVER_ERROR, Json(err_msg)).into_response()
            }
        }
    }
}
