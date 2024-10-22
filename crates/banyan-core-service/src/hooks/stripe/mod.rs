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
        (ET::ChargeSucceeded, EO::Charge(_)) => (), // this one fails to decode, expect errors
        (ET::PaymentIntentSucceeded, EO::PaymentIntent(_)) => (),
        (ET::PaymentIntentCreated, EO::PaymentIntent(_)) => (),

        // These are dynamic managed objects by our server, we don't need to know when we change
        // them
        (ET::PlanCreated, EO::Plan(_)) => (),
        (ET::PriceCreated, EO::Price(_)) => (),

        // We don't need to handle these yet, we're not allowing switching of plans through stripe,
        // and creation gets handled during the checkout confirmation
        (ET::CustomerSubscriptionCreated, EO::Subscription(_)) => (),
        (ET::CustomerSubscriptionUpdated, EO::Subscription(_)) => (),

        // Deletion events comes in at the end of a subscription cycle after a user has already
        // canceled, this is where we transition back to different subscription if desired.
        (ET::CustomerSubscriptionDeleted, EO::Subscription(sub)) => {
            subscription_events::deleted(&mut conn, sub).await?
        }

        // We don't support pausing and resuming our subscriptions
        (ET::CustomerSubscriptionPaused, EO::Subscription(_)) => (),
        (ET::CustomerSubscriptionResumed, EO::Subscription(_)) => (),

        (ET::InvoiceCreated, EO::Invoice(inv)) => {
            invoice_events::creation_handler(&mut conn, inv).await?
        }
        (ET::InvoiceUpcoming, EO::Invoice(inv)) => {
            invoice_events::creation_handler(&mut conn, inv).await?
        }

        (ET::InvoiceFinalizationFailed, EO::Invoice(inv)) => {
            invoice_events::update_handler(&mut conn, inv).await?
        }
        (ET::InvoiceFinalized, EO::Invoice(inv)) => {
            invoice_events::update_handler(&mut conn, inv).await?
        }
        (ET::InvoicePaid, EO::Invoice(inv)) => {
            invoice_events::update_handler(&mut conn, inv).await?
        }
        (ET::InvoicePaymentActionRequired, EO::Invoice(inv)) => {
            invoice_events::update_handler(&mut conn, inv).await?
        }
        (ET::InvoicePaymentFailed, EO::Invoice(inv)) => {
            invoice_events::update_handler(&mut conn, inv).await?
        }
        (ET::InvoicePaymentSucceeded, EO::Invoice(inv)) => {
            invoice_events::update_handler(&mut conn, inv).await?
        }
        (ET::InvoiceUpdated, EO::Invoice(inv)) => {
            invoice_events::update_handler(&mut conn, inv).await?
        }

        // This should be the only place where a subscription actually changes other than being
        // canceled / run out of paid time. This event indicates the customer has finished (and
        // payment has been confirmed) for a particular subscription.
        (ET::CheckoutSessionCompleted, EO::CheckoutSession(sess)) => {
            session_events::handler(&mut conn, sess).await?
        }

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

    #[error("stripe webhook payload was contained invalid data: {0}")]
    InvalidData(String),

    #[error("stripe webhook payload was missing required data: {0}")]
    MissingData(String),

    #[error("unable to locate associated data with webhook: {0}")]
    MissingTarget(String),
}

impl StripeWebhookError {
    pub fn invalid_data(val: &str) -> Self {
        StripeWebhookError::InvalidData(val.to_string())
    }

    pub fn missing_data(val: &str) -> Self {
        StripeWebhookError::MissingData(val.to_string())
    }

    pub fn missing_target(val: &str) -> Self {
        StripeWebhookError::MissingTarget(val.to_string())
    }
}

impl IntoResponse for StripeWebhookError {
    fn into_response(self) -> Response {
        match &self {
            StripeWebhookError::MissingData(_) | StripeWebhookError::InvalidData(_) => {
                tracing::error!("{self}");
                let err_msg = serde_json::json!({"msg": "missing expected data"});
                (StatusCode::BAD_REQUEST, Json(err_msg)).into_response()
            }
            StripeWebhookError::MissingTarget(_) => {
                tracing::error!("{self}");
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
