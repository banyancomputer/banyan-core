use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::database::models::{Invoice, InvoiceStatus};

#[derive(Deserialize, Serialize)]
pub struct ApiInvoice {
    id: String,
    amount_due: i64,
    status: InvoiceStatus,
    created_at: OffsetDateTime,
}

impl From<Invoice> for ApiInvoice {
    fn from(val: Invoice) -> Self {
        Self {
            id: val.id,
            amount_due: val.amount_due,
            status: val.status,
            created_at: val.created_at,
        }
    }
}