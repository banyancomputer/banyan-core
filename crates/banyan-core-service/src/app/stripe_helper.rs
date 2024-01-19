use crate::app::secrets::StripeSecret;
use crate::database::models::{StripeProduct, Subscription};
use crate::database::Database;

const BANDWIDTH_PRODUCT_KEY: &str = "bandwidth";

const PRODUCT_DESCRIPTOR_PREFIX: &str = "banyan";

const PRODUCT_METADATA_KEY: &str = "product-key";

const STORAGE_PRODUCT_KEY: &str = "storage";

const SUBSCRIPTION_METADATA_KEY: &str = "subscription-id";

pub struct StripeHelper {
    database: Database,
    client: stripe::Client,
}

impl StripeHelper {
    async fn find_or_register_product(
        &self,
        product_key: &str,
    ) -> Result<String, StripeHelperError> {
        let mut conn = self.database.begin().await?;
        let mut stripe_product = StripeProduct::from_product_key(&mut *conn, product_key).await?;

        // We've already created the product in stripe, return the existing product ID
        if let Some(stripe_product_id) = stripe_product.stripe_product_id {
            return Ok(stripe_product_id);
        }

        // Check if stripe already knows about this product
        if let Some(stripe_product_id) = search_products_for_key(&self.client, product_key).await? {
            stripe_product
                .record_stripe_product_id(&mut *conn, &stripe_product_id)
                .await?;
            conn.commit().await?;
            return Ok(stripe_product_id);
        }

        // It doesn't, we'll need to create a new one
        let new_product_id =
            register_stripe_product(&self.client, product_key, &stripe_product.title).await?;
        stripe_product
            .record_stripe_product_id(&mut *conn, &new_product_id)
            .await?;

        Ok(new_product_id)
    }

    async fn find_price_by_id(
        &self,
        price_id: &str,
    ) -> Result<Option<stripe::Price>, StripeHelperError> {
        use std::str::FromStr;

        use stripe::{Price, PriceId};

        let price_id = match PriceId::from_str(&price_id) {
            Ok(pid) => pid,
            Err(err) => {
                tracing::warn!("price ID stored in the database was an invalid format: {err}");
                // If this ever occurs we'll just overwrite the bad ID with a fresh one
                return Ok(None);
            }
        };

        match Price::retrieve(&self.client, &price_id, &["product"]).await {
            Ok(price) => Ok(Some(price)),
            Err(stripe::StripeError::Stripe(req_err)) if req_err.http_status == 404 => Ok(None),
            Err(err) => Err(StripeHelperError::from(err)),
        }
    }

    pub fn new(database: Database, stripe_secret: StripeSecret) -> Self {
        let client = stripe::Client::new(stripe_secret.key());
        Self { database, client }
    }

    async fn plan_price(
        &self,
        plan_product_id: &str,
        subscription: &Subscription,
    ) -> Result<stripe::Price, StripeHelperError> {
        use stripe::{
            CreatePrice, CreatePriceRecurring, CreatePriceRecurringInterval,
            CreatePriceRecurringUsageType, Currency, IdOrCreate, Metadata,
            PriceBillingScheme, PriceTaxBehavior,
        };

        // If we already have a cached price associated with this subscription, verify its still
        // valid on Stripe then return it, otherwise we'll create a new one.
        if let Some(plan_price_stripe_id) = &subscription.plan_price_stripe_id {
            if let Some(price) = self.find_price_by_id(&plan_price_stripe_id).await? {
                return Ok(price);
            }
        }

        let mut params = CreatePrice::new(Currency::USD);

        params.expand = &["product"];
        params.product = Some(IdOrCreate::Id(plan_product_id));

        let price = subscription
            .plan_base_price
            .as_ref()
            .ok_or(StripeHelperError::MissingPrice)?;

        // Assign the base price of the plan to the stripe price, and the billing scheme
        params.unit_amount = Some(price.in_cents());
        params.billing_scheme = Some(PriceBillingScheme::PerUnit);

        // Set the nature of our payment (recurring monthly, billed monthly subscription)
        params.recurring = Some(CreatePriceRecurring {
            interval: CreatePriceRecurringInterval::Month,
            interval_count: Some(1),
            usage_type: Some(CreatePriceRecurringUsageType::Licensed),
            ..Default::default()
        });

        // Tax related settings need to be set as well
        params.tax_behavior = Some(PriceTaxBehavior::Exclusive);

        params.metadata = Some(Metadata::from([(
            SUBSCRIPTION_METADATA_KEY.to_string(),
            subscription.id.to_string(),
        )]));

        todo!()
    }

    pub async fn realize_subscription(
        &self,
        user_id: &str,
        subscription: &mut Subscription,
    ) -> Result<Option<stripe::Subscription>, StripeHelperError> {
        let plan_product_key = format!("{}-plan", subscription.service_key);

        let plan_product_id = self.find_or_register_product(&plan_product_key).await?;
        let _plan_price = self.plan_price(&plan_product_id, &subscription).await?;

        let _bandwidth_product_id = self
            .find_or_register_product(&BANDWIDTH_PRODUCT_KEY)
            .await?;
        let _storage_product_id = self.find_or_register_product(&STORAGE_PRODUCT_KEY).await?;

        todo!()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StripeHelperError {
    #[error("failure while querying the database: {0}")]
    DatabaseFailure(#[from] sqlx::Error),

    #[error("attempted to create price for subscription without an available price")]
    MissingPrice,

    #[error("failure in making a request to the stripe API: {0}")]
    StripeClientError(#[from] stripe::StripeError),
}

async fn register_stripe_product(
    client: &stripe::Client,
    product_key: &str,
    title: &str,
) -> Result<String, StripeHelperError> {
    use stripe::{CreateProduct, Metadata, Product, ProductType};

    let descriptor = format!("{}-{}", PRODUCT_DESCRIPTOR_PREFIX, product_key).to_uppercase();
    let metadata = Metadata::from([(PRODUCT_METADATA_KEY.to_string(), product_key.to_string())]);

    let mut product_details = CreateProduct::new(title);
    product_details.shippable = Some(false);
    product_details.type_ = Some(ProductType::Service);
    product_details.statement_descriptor = Some(&descriptor);
    product_details.metadata = Some(metadata);

    let new_product = Product::create(&client, product_details).await?;

    Ok(new_product.id.to_string())
}

async fn search_products_for_key(
    client: &stripe::Client,
    product_key: &str,
) -> Result<Option<String>, StripeHelperError> {
    use stripe::{ListProducts, Product};

    let search_params = ListProducts {
        active: Some(true),
        ..Default::default()
    };

    let existing_products = Product::list(&client, &search_params).await?;
    for product in existing_products.data.iter() {
        let metadata = match &product.metadata {
            Some(m) => m,
            None => continue,
        };

        if let Some(key) = metadata.get(PRODUCT_METADATA_KEY) {
            if key == product_key {
                return Ok(Some(product.id.to_string()));
            }
        }
    }

    Ok(None)
}
