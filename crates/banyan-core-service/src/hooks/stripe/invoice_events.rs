use time::OffsetDateTime;

use crate::app::stripe_helper::{METADATA_SUBSCRIPTION_KEY, METADATA_USER_KEY};
use crate::database::models::{Invoice, InvoiceStatus, NewInvoice, PriceUnits};
use crate::database::DatabaseConnection;
use crate::hooks::stripe::StripeWebhookError;

pub async fn creation_handler(
    conn: &mut DatabaseConnection,
    stripe_invoice: &stripe::Invoice,
) -> Result<(), StripeWebhookError> {
    let stripe_invoice_id = stripe_invoice.id.to_string();
    let stripe_customer_id = stripe_invoice
        .customer
        .as_ref()
        .ok_or(StripeWebhookError::MissingData)?
        .id()
        .to_string();

    let total_amount = stripe_invoice
        .amount_due
        .map(PriceUnits::from_cents)
        .ok_or(StripeWebhookError::MissingData)?;

    let invoice_status = stripe_invoice
        .status
        .map(InvoiceStatus::from)
        .ok_or(StripeWebhookError::MissingData)?;

    let period_start = stripe_invoice.period_start.ok_or(StripeWebhookError::MissingData)?;
    let billing_start = OffsetDateTime::from_unix_timestamp(period_start).map_err(|_| StripeWebhookError::InvalidData)?;
    let period_end = stripe_invoice.period_end.ok_or(StripeWebhookError::MissingData)?;
    let billing_end = OffsetDateTime::from_unix_timestamp(period_end).map_err(|_| StripeWebhookError::InvalidData)?;

    let stripe_metadata = stripe_invoice
        .metadata
        .as_ref()
        .ok_or(StripeWebhookError::MissingData)?;
    let m_user_id = stripe_metadata
        .get(METADATA_USER_KEY)
        .ok_or(StripeWebhookError::MissingData)?;
    let m_subscription_id = stripe_metadata
        .get(METADATA_SUBSCRIPTION_KEY)
        .ok_or(StripeWebhookError::MissingData)?;

    NewInvoice {
        user_id: &m_user_id,

        stripe_invoice_id: &stripe_invoice_id,
        stripe_customer_id: &stripe_customer_id,

        billing_start: &billing_start,
        billing_end: &billing_end,

        subscription_id: &m_subscription_id,

        total_amount,
        status: invoice_status,
    }
    .save(&mut *conn)
    .await?;

    Ok(())
}

pub async fn update_handler(
    conn: &mut DatabaseConnection,
    invoice: &stripe::Invoice,
) -> Result<(), StripeWebhookError> {
    let invoice_id = invoice.id.to_string();
    let new_status = invoice
        .status
        .map(InvoiceStatus::from)
        .ok_or(StripeWebhookError::MissingData)?;

    let mut invoice = Invoice::from_stripe_invoice_id(&mut *conn, &invoice_id)
        .await?
        .ok_or(StripeWebhookError::MissingTarget)?;
    invoice.update_status(&mut *conn, new_status).await?;

    todo!()
}
