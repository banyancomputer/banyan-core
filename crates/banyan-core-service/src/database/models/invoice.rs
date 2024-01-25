use time::OffsetDateTime;

use crate::database::models::{InvoiceStatus, PriceUnits};
use crate::database::DatabaseConnection;

pub struct NewInvoice<'a> {
    pub user_id: &'a str,

    pub stripe_customer_id: &'a str,
    pub stripe_invoice_id: &'a str,

    pub billing_start: &'a OffsetDateTime,
    pub billing_end: &'a OffsetDateTime,

    pub subscription_id: &'a str,

    pub total_amount: PriceUnits,
    pub status: InvoiceStatus,
}

impl<'a> NewInvoice<'a> {
    pub async fn save(self, conn: &mut DatabaseConnection) -> Result<String, sqlx::Error> {
        let now = OffsetDateTime::now_utc();

        sqlx::query_scalar!(
            r#"INSERT INTO invoices (user_id, stripe_customer_id, stripe_invoice_id, billing_start,
                   billing_end, subscription_id, total_amount, status, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9)
                 RETURNING id;"#,
            self.user_id,
            self.stripe_customer_id,
            self.stripe_invoice_id,
            self.billing_start,
            self.billing_end,
            self.subscription_id,
            self.total_amount,
            self.status,
            now,
        )
        .fetch_one(&mut *conn)
        .await
    }
}

#[derive(sqlx::FromRow)]
pub struct Invoice {
    pub id: String,

    pub billing_start: OffsetDateTime,
    pub billing_end: OffsetDateTime,

    pub subscription_id: String,

    pub total_amount: PriceUnits,
    pub status: InvoiceStatus,

    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl Invoice {
    pub async fn from_stripe_invoice_id(
        conn: &mut DatabaseConnection,
        stripe_invoice_id: &str,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"SELECT id, billing_start, billing_end, subscription_id,
                   total_amount as 'total_amount: PriceUnits', status as 'status: InvoiceStatus',
                   created_at, updated_at
                 FROM invoices
                 WHERE stripe_invoice_id = $1;"#,
            stripe_invoice_id,
        )
        .fetch_optional(&mut *conn)
        .await
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
