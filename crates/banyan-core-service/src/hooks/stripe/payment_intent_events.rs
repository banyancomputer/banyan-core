use crate::database::models::{Invoice, StripePaymentIntentStatus};
use crate::database::DatabaseConnection;
use crate::hooks::stripe::StripeWebhookError;

pub async fn update_status(
    conn: &mut DatabaseConnection,
    payment_intent: &stripe::PaymentIntent,
) -> Result<(), StripeWebhookError> {
    let payment_intent_id = payment_intent.id.to_string();

    let intent_status = StripePaymentIntentStatus::from(payment_intent.status);
    let mut invoice = Invoice::from_payment_intent_id(&mut *conn, &payment_intent_id)
        .await?
        .ok_or(StripeWebhookError::MissingTarget)?;
    invoice
        .update_intent_status(&mut *conn, intent_status)
        .await?;

    Ok(())
}
