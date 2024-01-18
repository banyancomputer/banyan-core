use crate::database::Database;
use crate::database::models::Subscription;
use crate::app::secrets::StripeSecret;

pub struct StripeHelper {
    database: Database,
    stripe_secret: StripeSecret,
}

impl StripeHelper {
    fn find_or_register_product(&self, product_key: &str) -> Result<String, StripeHelperError> {
        todo!()
    }

    pub fn new(database: Database, stripe_secret: StripeSecret) -> Self {
        Self { database, stripe_secret }
    }

    pub async fn realize_subscription(
        &self,
        user_id: &str,
        subscription: &Subscription,
    ) -> Result<(), StripeHelperError> {
        // We don't need to do anything for the starter
        if subscription.service_key == "starter" {
            return Ok(());
        }

        let base_product_key = &subscription.service_key;

        todo!()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StripeHelperError {
    #[error("failure while querying the database: {0}")]
    DatabaseFailure(#[from] sqlx::Error),
}
