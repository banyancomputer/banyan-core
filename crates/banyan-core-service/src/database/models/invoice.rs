use time::OffsetDateTime;

use crate::database::DatabaseConnection;
use crate::database::models::{InvoiceStatus, StripePaymentIntentStatus};

pub struct NewInvoice<'a> {
    pub user_id: &'a str,

    pub stripe_customer_id: &'a str,
    pub stripe_invoice_id: &'a str,

    pub amount_due: i64,
    pub status: InvoiceStatus,

    pub stripe_payment_intent_id: &'a str,
}

impl<'a> NewInvoice<'a> {
    pub async fn save(self, conn: &mut DatabaseConnection) -> Result<String, sqlx::Error> {
        let now = OffsetDateTime::now_utc();

        sqlx::query_scalar!(
            r#"INSERT INTO invoices (user_id, stripe_customer_id, stripe_invoice_id, amount_due,
                   status, stripe_payment_intent_id, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7)
                 RETURNING id;"#,
            self.user_id,
            self.stripe_customer_id,
            self.stripe_invoice_id,
            self.amount_due,
            self.status,
            self.stripe_payment_intent_id,
            now,
        )
        .fetch_one(&mut *conn)
        .await
    }
}

#[derive(sqlx::FromRow)]
pub struct Invoice {
    id: String,

    user_id: String,

    stripe_customer_id: String,
    stripe_invoice_id: String,

    amount_due: i64,
    status: InvoiceStatus,

    stripe_payment_intent_id: String,
    stripe_payment_intent_status: Option<StripePaymentIntentStatus>,

    created_at: OffsetDateTime,
}
