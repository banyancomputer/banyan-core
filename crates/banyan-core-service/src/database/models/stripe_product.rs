use crate::database::models::TaxClass;
use crate::database::DatabaseConnection;

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
pub struct StripeProduct {
    pub id: String,

    pub product_key: String,
    pub tax_class: TaxClass,
    pub title: String,

    pub stripe_product_id: Option<String>,
}

impl StripeProduct {
    pub async fn from_product_key(
        conn: &mut DatabaseConnection,
        product_key: &str,
        tax_class: TaxClass,
    ) -> Result<Self, sqlx::Error> {
        sqlx::query_as!(
            Self,
            r#"SELECT id, product_key, tax_class as 'tax_class: TaxClass', title, stripe_product_id
                   FROM stripe_products
                WHERE product_key = $1 AND tax_class = $2;"#,
            product_key,
            tax_class,
        )
        .fetch_one(&mut *conn)
        .await
    }

    pub async fn record_stripe_product_id(
        &mut self,
        conn: &mut DatabaseConnection,
        stripe_product_id: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE stripe_products SET stripe_product_id = $1 WHERE id = $2;",
            stripe_product_id,
            self.id,
        )
        .execute(&mut *conn)
        .await?;

        self.stripe_product_id = Some(stripe_product_id.to_string());

        Ok(())
    }
}
