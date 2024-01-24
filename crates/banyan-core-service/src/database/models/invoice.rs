use time::OffsetDateTime;

use crate::database::models::{InvoiceStatus, PriceUnits, StripePaymentIntentStatus};
use crate::database::DatabaseConnection;

pub struct NewInvoice<'a> {
    pub user_id: &'a str,

    pub stripe_customer_id: &'a str,
    pub stripe_invoice_id: &'a str,

    pub amount_due: PriceUnits,
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
    pub id: String,

    pub amount_due: PriceUnits,
    pub status: InvoiceStatus,

    pub created_at: OffsetDateTime,
}

impl Invoice {
    pub async fn from_payment_intent_id(
        conn: &mut DatabaseConnection,
        stripe_payment_intent_id: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"SELECT id, amount_due as 'amount_due: PriceUnits', status as 'status: InvoiceStatus', created_at
                 FROM invoices
                 WHERE stripe_payment_intent_id = $1;"#,
            stripe_payment_intent_id,
        )
        .fetch_optional(&mut *conn)
        .await
    }

    pub async fn from_stripe_invoice_id(
        conn: &mut DatabaseConnection,
        stripe_invoice_id: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"SELECT id, amount_due as 'amount_due: PriceUnits', status as 'status: InvoiceStatus', created_at
                 FROM invoices
                 WHERE stripe_invoice_id = $1;"#,
            stripe_invoice_id,
        )
        .fetch_optional(&mut *conn)
        .await
    }

    pub async fn update_intent_status(
        &mut self,
        conn: &mut DatabaseConnection,
        status: StripePaymentIntentStatus,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE invoices SET stripe_payment_intent_status = $1 WHERE id = $2;",
            status,
            self.id,
        )
        .execute(&mut *conn)
        .await?;

        Ok(())
    }

    pub async fn update_status(
        &mut self,
        conn: &mut DatabaseConnection,
        status: InvoiceStatus,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE invoices SET status = $1 WHERE id = $2;",
            status,
            self.id,
        )
        .execute(&mut *conn)
        .await?;

        self.status = status;

        Ok(())
    }
}
