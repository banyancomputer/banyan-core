use crate::database::DatabaseConnection;
use crate::hooks::stripe::StripeWebhookError;

pub async fn handler(
    conn: &mut DatabaseConnection,
    session: &stripe::CheckoutSession,
) -> Result<(), StripeWebhookError> {
    let _session_str = session.id.to_string();

    let _subscription = session.subscription.as_ref().ok_or(StripeWebhookError::MissingData)?;
    let _invoice = session.invoice.as_ref().ok_or(StripeWebhookError::MissingData)?;
    let _metadata = session.metadata.as_ref().ok_or(StripeWebhookError::MissingData)?;

    // we should receive an invoice with a payment_status of Paid
    // get subscription.id for stripe_subscription_id

    Ok(())
}
