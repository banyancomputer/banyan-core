use time::OffsetDateTime;

use crate::database::models::{InvoiceStatus, StripePaymentIntentStatus};

pub struct NewInvoice<'a> {
    user_id: &'a str,

    stripe_customer_id: &'a str,
    stripe_invoice_id: &'a str,

    amount_due: i64,
    billing_reason: &'a str,
    status: InvoiceStatus,

    stripe_payment_intent_id: Option<&'a str>,
    stripe_payment_intent_status: Option<&'a StripePaymentIntentStatus>,
}

#[derive(sqlx::FromRow)]
pub struct Invoice {
    id: String,

    user_id: String,

    stripe_customer_id: String,
    stripe_invoice_id: String,

    amount_due: i64,
    billing_reason: String,
    status: InvoiceStatus,

    stripe_payment_intent_id: Option<String>,
    stripe_payment_intent_status: Option<StripePaymentIntentStatus>,

    created_at: OffsetDateTime,
}
