use crate::database::models::{Invoice, InvoiceStatus, NewInvoice, PriceUnits};
use crate::database::DatabaseConnection;
use crate::hooks::stripe::StripeWebhookError;

pub async fn creation_handler(
    _conn: &mut DatabaseConnection,
    invoice: &stripe::Invoice,
) -> Result<(), StripeWebhookError> {
    let customer = invoice
        .customer
        .clone()
        .ok_or(StripeWebhookError::MissingData)?;
    let _customer_id = customer.id().to_string();

    tracing::info!("new invoice creation event: {invoice:?}");

    //let user_id = sqlx::query_scalar!(
    //    "SELECT id FROM users WHERE stripe_customer_id = $1;",
    //    customer_id,
    //)
    //.fetch_optional(&mut *conn)
    //.await?
    //.ok_or(StripeWebhookError::MissingTarget)?;

    let stripe_invoice_id = invoice.id.to_string();
    let total_amount = invoice.amount_due.map(PriceUnits::from_cents).ok_or(StripeWebhookError::MissingData)?;
    //let invoice_status = invoice
    //    .status
    //    .map(InvoiceStatus::from)
    //    .ok_or(StripeWebhookError::MissingData)?;

    //NewInvoice {
    //    user_id: &user_id,

    //    stripe_customer_id: &customer_id,
    //    stripe_invoice_id: &stripe_invoice_id,

    //    billing_start: &billing_start,
    //    billing_end: &billing_end,

    //    subscription_id: &subscription_id,

    //    total_amount: invoice_amt,
    //    status: invoice_status,

    //    stripe_payment_intent_id: &invoice_id,
    //}
    //.save(&mut *conn)
    //.await?;

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
