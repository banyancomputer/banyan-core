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
    tracing::warn!("create handler for invoice:{stripe_invoice_id}");

    let stripe_customer_id = stripe_invoice
        .customer
        .as_ref()
        .ok_or(StripeWebhookError::missing_data("invoice_create/customer/id"))?
        .id()
        .to_string();

    let total_amount = stripe_invoice
        .amount_due
        .map(PriceUnits::from_cents)
        .ok_or(StripeWebhookError::missing_data("invoice_create/total_amount"))?;
    let invoice_status = stripe_invoice
        .status
        .map(InvoiceStatus::from)
        .ok_or(StripeWebhookError::missing_data("invoice_create/status"))?;

    let period_start = stripe_invoice.period_start.ok_or(StripeWebhookError::missing_data("invoice_create/period_start"))?;
    let billing_start = OffsetDateTime::from_unix_timestamp(period_start).map_err(|_| StripeWebhookError::invalid_data("invoice_create/billing_start"))?;
    let period_end = stripe_invoice.period_end.ok_or(StripeWebhookError::missing_data("invoice_create/period_end"))?;
    let billing_end = OffsetDateTime::from_unix_timestamp(period_end).map_err(|_| StripeWebhookError::invalid_data("invoice_create/billing_end"))?;

    let stripe_metadata = stripe_invoice
        .metadata
        .as_ref()
        .ok_or(StripeWebhookError::missing_data("invoice_create/meta"))?;
    let m_user_id = stripe_metadata
        .get(METADATA_USER_KEY)
        .ok_or(StripeWebhookError::missing_data("invoice_create/meta/db_user_id"))?;
    let m_subscription_id = stripe_metadata
        .get(METADATA_SUBSCRIPTION_KEY)
        .ok_or(StripeWebhookError::missing_data("invoice_create/meta/db_subscription_id"))?;

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
    stripe_invoice: &stripe::Invoice,
) -> Result<(), StripeWebhookError> {
    let stripe_invoice_id = stripe_invoice.id.to_string();
    tracing::warn!("update handler for invoice:{stripe_invoice_id}");

    let new_status = stripe_invoice
        .status
        .map(InvoiceStatus::from)
        .ok_or(StripeWebhookError::missing_data("invoice_update/status"))?;

    let mut invoice = Invoice::from_stripe_invoice_id(&mut *conn, &stripe_invoice_id)
        .await?
        .ok_or(StripeWebhookError::missing_target("db_invoice"))?;
    invoice.update_status(&mut *conn, new_status).await?;

    Ok(())
}
