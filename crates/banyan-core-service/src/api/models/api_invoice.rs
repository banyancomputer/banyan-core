use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::database::models::{Invoice, InvoiceStatus};

#[derive(Deserialize, Serialize)]
pub struct ApiInvoice {
    id: String,

    #[serde(with = "time::serde::rfc3339")]
    billing_start: OffsetDateTime,

    #[serde(with = "time::serde::rfc3339")]
    billing_end: OffsetDateTime,

    subscription_id: String,

    total_amount: i64,
    status: InvoiceStatus,

    #[serde(with = "time::serde::rfc3339")]
    created_at: OffsetDateTime,
}

impl From<Invoice> for ApiInvoice {
    fn from(val: Invoice) -> Self {
        Self {
            id: val.id,

            billing_start: val.billing_start,
            billing_end: val.billing_end,

            subscription_id: val.subscription_id,

            total_amount: val.total_amount.in_cents(),
            status: val.status,

            created_at: val.created_at,
        }
    }
}
