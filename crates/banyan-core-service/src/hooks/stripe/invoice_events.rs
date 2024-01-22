use crate::database::DatabaseConnection;
use crate::database::models::{Invoice, InvoiceStatus, NewInvoice};

use crate::hooks::stripe::StripeWebhookError;

pub async fn created(conn: &mut DatabaseConnection, invoice: &stripe::Invoice) -> Result<(), StripeWebhookError> {
    let customer = invoice.customer.clone().ok_or(StripeWebhookError::MissingData)?;
    let customer_id = customer.id().to_string();

    let user_id = sqlx::query_scalar!(
        "SELECT id FROM users WHERE stripe_customer_id = $1;",
        customer_id,
    )
    .fetch_optional(&mut *conn)
    .await?
    .ok_or(StripeWebhookError::MissingTarget)?;

    let invoice_id = invoice.id.to_string();
    let invoice_amt = invoice.amount_due.ok_or(StripeWebhookError::MissingData)?;
    let invoice_status = invoice.status.map(InvoiceStatus::from).ok_or(StripeWebhookError::MissingData)?;

    NewInvoice {
        user_id: &user_id,

        stripe_customer_id: &customer_id,
        stripe_invoice_id: &invoice_id,

        amount_due: invoice_amt,
        status: invoice_status,

        stripe_payment_intent_id: &invoice_id,
    }
    .save(&mut *conn)
    .await?;

    Ok(())
}

pub async fn status_update(conn: &mut DatabaseConnection, invoice: &stripe::Invoice) -> Result<(), StripeWebhookError> {
    let invoice_id = invoice.id.to_string();
    let new_status = invoice.status.map(InvoiceStatus::from).ok_or(StripeWebhookError::MissingData)?;

    let mut invoice = Invoice::from_stripe_invoice_id(&mut *conn, &invoice_id).await?.ok_or(StripeWebhookError::MissingTarget)?;
    invoice.update_status(&mut *conn, new_status).await?;

    Ok(())
}
