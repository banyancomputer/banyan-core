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

    tracing::info!("stripe_event:{event:?}");
    // Order is:
    //
    // * charge.succeeded
    // * customer.subscription.created
    // * customer.subscription.updated
    // * payment_intent.succeeded
    // * payment_intent.created
    // * invoice.created
    // * invoice.finalized
    // * invoice.updated
    // * invoice.paid
    // * invoice.payment_succeeded
    // * checkout.session.completed

    use {EventObject as EO, EventType as ET};

    match (event.type_, &event.data.object) {
        // We don't need to handle, but don't want to log
        (ET::CustomerCreated, EO::Customer(_)) => (),
        (ET::InvoiceUpcoming, EO::Invoice(_)) => (),

        (EType::InvoiceCreated, EO::Invoice(invoice)) => {
            invoice_events::created(&mut conn, invoice).await?
        }
        (EType::InvoiceFinalizationFailed, EO::Invoice(invoice)) => {
            invoice_events::status_update(&mut conn, invoice).await?
        }
        (EType::InvoiceFinalized, EO::Invoice(invoice)) => {
            invoice_events::status_update(&mut conn, invoice).await?
        }
        (EType::InvoicePaid, EO::Invoice(invoice)) => {
            invoice_events::status_update(&mut conn, invoice).await?
        }
        (EType::InvoicePaymentActionRequired, EO::Invoice(invoice)) => {
            invoice_events::status_update(&mut conn, invoice).await?
        }
        (EType::InvoicePaymentFailed, EO::Invoice(invoice)) => {
            invoice_events::status_update(&mut conn, invoice).await?
        }
        // This one should probably be handled as a special case as we can glean some extra data
        // from it, but its not a high priority
        (EType::InvoiceUpdated, EO::Invoice(invoice)) => {
            invoice_events::status_update(&mut conn, invoice).await?
        }

        (EType::CheckoutSessionCompleted, EO::CheckoutSession(sess)) => {
            checkout_session_events::completed(&mut conn, sess).await?
        }
        (EType::CheckoutSessionExpired, EO::CheckoutSession(sess)) => {
            checkout_session_events::expired(&mut conn, sess).await?
        }

        (EType::CustomerSubscriptionCreated, EO::Subscription(subscription)) => {
            customer_subscription_events::manage(&mut conn, subscription).await?
        }
        (EType::CustomerSubscriptionDeleted, EO::Subscription(subscription)) => {
            customer_subscription_events::manage(&mut conn, subscription).await?
        }
        (EType::CustomerSubscriptionPaused, EO::Subscription(subscription)) => {
            customer_subscription_events::manage(&mut conn, subscription).await?
        }
        (EType::CustomerSubscriptionResumed, EO::Subscription(subscription)) => {
            customer_subscription_events::manage(&mut conn, subscription).await?
        }
        (EType::CustomerSubscriptionUpdated, EO::Subscription(subscription)) => {
            customer_subscription_events::manage(&mut conn, subscription).await?
        }

        (EType::PaymentIntentCreated, EO::PaymentIntent(intent)) => {
            payment_intent_events::update_status(&mut conn, intent).await?
        }
        (EType::PaymentIntentSucceeded, EO::PaymentIntent(intent)) => {
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
