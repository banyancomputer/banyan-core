use time::OffsetDateTime;
use url::Url;

use crate::app::secrets::StripeSecrets;
use crate::database::models::{StripeProduct, Subscription, TaxClass, User};
use crate::database::Database;
use crate::pricing::SUBSCRIPTION_CHANGE_EXPIRATION_WINDOW;

const BANDWIDTH_PRODUCT_KEY: &str = "bandwidth";

const METADATA_PRODUCT_KEY: &str = "product-key";

const METADATA_SUBSCRIPTION_KEY: &str = "subscription-id";

const METADATA_USER_KEY: &str = "user-id";

const PRODUCT_DESCRIPTOR_PREFIX: &str = "banyan";

const PRODUCT_TAXCLASS_KEY: &str = "tax-class";

const STORAGE_PRODUCT_KEY: &str = "storage";

pub struct StripeHelper {
    database: Database,
    client: stripe::Client,
}

impl StripeHelper {
    async fn bandwidth_price(
        &self,
        bandwidth_product_id: &str,
        subscription: &mut Subscription,
    ) -> Result<stripe::Price, StripeHelperError> {
        use stripe::{
            CreatePrice, CreatePriceRecurring, CreatePriceRecurringAggregateUsage,
            CreatePriceRecurringInterval, CreatePriceRecurringUsageType, CreatePriceTiers,
            Currency, IdOrCreate, Metadata, Price, PriceBillingScheme, PriceTaxBehavior,
            PriceTiersMode, UpTo, UpToOther,
        };

        // If we already have a cached price associated with bandwidth pricing, verify its still
        // valid on Stripe then return it, otherwise we'll create a new one.
        if let Some(stripe_price_id) = &subscription.bandwidth_stripe_price_id {
            if let Some(price) = self.find_price_by_id(stripe_price_id).await? {
                return Ok(price);
            }
        }

        let mut params = CreatePrice::new(Currency::USD);

        params.expand = &["product"];
        params.product = Some(IdOrCreate::Id(bandwidth_product_id));

        let price = subscription
            .bandwidth_price
            .as_ref()
            .ok_or(StripeHelperError::MissingPrice)?;

        params.billing_scheme = Some(PriceBillingScheme::Tiered);
        params.tiers_mode = Some(PriceTiersMode::Graduated);
        params.tiers = Some(vec![
            CreatePriceTiers {
                flat_amount: Some(0),
                up_to: Some(UpTo::Max(subscription.included_bandwidth as u64)),
                ..Default::default()
            },
            CreatePriceTiers {
                unit_amount_decimal: Some(price.in_fractional_cents()),
                up_to: Some(UpTo::Other(UpToOther::Inf)),
                ..Default::default()
            },
        ]);

        // Set the nature of our payment (recurring monthly, billed monthly subscription)
        params.recurring = Some(CreatePriceRecurring {
            aggregate_usage: Some(CreatePriceRecurringAggregateUsage::Sum),
            interval: CreatePriceRecurringInterval::Month,
            interval_count: Some(1),
            usage_type: Some(CreatePriceRecurringUsageType::Metered),
            ..Default::default()
        });

        // Tax related settings need to be set as well
        params.tax_behavior = Some(PriceTaxBehavior::Exclusive);

        params.metadata = Some(Metadata::from([(
            METADATA_SUBSCRIPTION_KEY.to_string(),
            subscription.id.to_string(),
        )]));

        let price = Price::create(&self.client, params).await?;

        let mut conn = self.database.acquire().await?;
        subscription
            .persist_bandwidth_price_stripe_id(&mut conn, price.id.as_str())
            .await?;

        Ok(price)
    }

    pub async fn checkout(
        &self,
        base_url: &Url,
        user: &mut User,
        subscription: &mut Subscription,
    ) -> Result<stripe::CheckoutSession, StripeHelperError> {
        use stripe::{
            CheckoutSession, CheckoutSessionMode, CreateCheckoutSession,
            CreateCheckoutSessionSubscriptionData, Currency, Metadata,
        };

        let customer = self.find_or_create_customer(&mut *user).await?;
        let mut params = CreateCheckoutSession::new();

        params.customer = Some(customer.id);
        // When we develop a tax obligation
        //params.automatic_tax = Some(CreateCheckoutSessionAutomaticTax { enabled: true });
        params.currency = Some(Currency::USD);
        params.mode = Some(CheckoutSessionMode::Subscription);

        let expiration = OffsetDateTime::now_utc() + SUBSCRIPTION_CHANGE_EXPIRATION_WINDOW;
        params.expires_at = Some(expiration.unix_timestamp());

        let line_items = self.subscription_items(&mut *subscription).await?;
        params.line_items = Some(line_items);

        let metadata = Metadata::from([
            (
                METADATA_SUBSCRIPTION_KEY.to_string(),
                subscription.id.clone(),
            ),
            (
                METADATA_USER_KEY.to_string(),
                user.id.to_string(),
            ),
        ]);

        params.metadata = Some(metadata.clone());
        params.subscription_data = Some(CreateCheckoutSessionSubscriptionData {
            description: Some(format!("Banyan Storage - {}", subscription.title)),
            metadata: Some(metadata),
            ..Default::default()
        });

        params.metadata = Some(Metadata::from([
            (
                METADATA_SUBSCRIPTION_KEY.to_string(),
                subscription.id.clone(),
            ),
            (
                METADATA_USER_KEY.to_string(),
                user.id.to_string(),
            ),
        ]));

        // When a user cancel's just send them back to the app, we could trakc and associate these
        // but we won't get the specific checkout session id
        let mut cancellation_url = base_url.clone();
        cancellation_url.set_path("/api/v1/subscriptions/cancel");
        params.cancel_url = Some(cancellation_url.as_str());

        let mut success_base = base_url.clone();
        success_base.set_path("/api/v1/subscriptions/success");
        let success_url = format!("{success_base}/{{CHECKOUT_SESSION_ID}}");
        params.success_url = Some(success_url.as_str());

        let checkout_session = CheckoutSession::create(&self.client, params).await?;

        Ok(checkout_session)
    }

    async fn find_or_register_product(
        &self,
        product_key: &str,
        tax_class: TaxClass,
    ) -> Result<String, StripeHelperError> {
        let mut conn = self.database.begin().await?;
        let mut product =
            StripeProduct::from_product_key(&mut conn, product_key, tax_class).await?;

        // We've already created the product in stripe, return the existing product ID
        if let Some(stripe_product_id) = product.stripe_product_id {
            return Ok(stripe_product_id);
        }

        // Check if stripe already knows about this product
        if let Some(stripe_product_id) =
            search_products_for_key(&self.client, product_key, tax_class).await?
        {
            product
                .record_stripe_product_id(&mut conn, &stripe_product_id)
                .await?;
            conn.commit().await?;
            return Ok(stripe_product_id);
        }

        // It doesn't, we'll need to create a new one
        let new_product =
            register_stripe_product(&self.client, product_key, tax_class, &product.title).await?;
        let new_product_id = new_product.id.as_str().to_string();

        product
            .record_stripe_product_id(&mut conn, &new_product_id)
            .await?;

        Ok(new_product_id)
    }

    async fn find_or_create_customer(
        &self,
        user: &mut User,
    ) -> Result<stripe::Customer, StripeHelperError> {
        use stripe::{CreateCustomer, Customer, Metadata};

        // If we already have a cached customer associated with the user, validate it still exists
        // in Stripe and if it is valid, return it directly, otherwise we'll create a new one.
        if let Some(cust_id) = &user.stripe_customer_id {
            if let Some(customer) = self.find_customer_by_id(cust_id).await? {
                return Ok(customer);
            }
        }

        let mut params = CreateCustomer::new();

        params.email = Some(user.email.as_str());
        params.name = Some(user.display_name.as_str());
        params.metadata = Some(Metadata::from([(
            METADATA_USER_KEY.to_string(),
            user.id.clone(),
        )]));

        let customer = Customer::create(&self.client, params).await?;

        let mut conn = self.database.acquire().await?;
        user.persist_customer_stripe_id(&mut conn, customer.id.as_str())
            .await?;

        Ok(customer)
    }

    async fn find_customer_by_id(
        &self,
        customer_id: &str,
    ) -> Result<Option<stripe::Customer>, StripeHelperError> {
        use std::str::FromStr;

        use stripe::{Customer, CustomerId};

        let customer_id = match CustomerId::from_str(customer_id) {
            Ok(cid) => cid,
            Err(err) => {
                tracing::warn!("customer ID stored in the database was an invalid format: {err}");
                // If this ever occurs we'll just overwrite the bad ID with a fresh one
                return Ok(None);
            }
        };

        match Customer::retrieve(&self.client, &customer_id, &[]).await {
            Ok(cust) => Ok(Some(cust)),
            Err(stripe::StripeError::Stripe(req_err)) if req_err.http_status == 404 => Ok(None),
            Err(err) => Err(StripeHelperError::from(err)),
        }
    }

    async fn find_price_by_id(
        &self,
        price_id: &str,
    ) -> Result<Option<stripe::Price>, StripeHelperError> {
        use std::str::FromStr;

        use stripe::{Price, PriceId};

        let price_id = match PriceId::from_str(price_id) {
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

    pub fn new(database: Database, stripe_secret: StripeSecrets) -> Self {
        let client = stripe::Client::new(stripe_secret.secret());
        Self { database, client }
    }

    async fn plan_price(
        &self,
        plan_product_id: &str,
        subscription: &mut Subscription,
    ) -> Result<stripe::Price, StripeHelperError> {
        use stripe::{
            CreatePrice, CreatePriceRecurring, CreatePriceRecurringInterval,
            CreatePriceRecurringUsageType, Currency, IdOrCreate, Metadata, Price,
            PriceBillingScheme, PriceTaxBehavior,
        };

        // If we already have a cached price associated with this subscription, verify its still
        // valid on Stripe then return it, otherwise we'll create a new one.
        if let Some(price_stripe_id) = &subscription.plan_price_stripe_id {
            if let Some(price) = self.find_price_by_id(price_stripe_id).await? {
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
            METADATA_SUBSCRIPTION_KEY.to_string(),
            subscription.id.to_string(),
        )]));

        let price = Price::create(&self.client, params).await?;

        let mut conn = self.database.acquire().await?;
        subscription
            .persist_plan_price_stripe_id(&mut conn, price.id.as_str())
            .await?;

        Ok(price)
    }

    pub async fn portal_session(
        &self,
        base_url: &Url,
        customer_id: &str,
    ) -> Result<stripe::BillingPortalSession, StripeHelperError> {
        use std::str::FromStr;

        use stripe::{BillingPortalSession, CreateBillingPortalSession, CustomerId};

        let customer_id = CustomerId::from_str(customer_id)?;
        let mut params = CreateBillingPortalSession::new(customer_id);

        let mut return_url = base_url.clone();
        return_url.set_path("/");
        params.return_url = Some(return_url.as_str());

        let billing_portal_session = BillingPortalSession::create(&self.client, params).await?;

        Ok(billing_portal_session)
    }

    pub async fn subscription_items(
        &self,
        subscription: &mut Subscription,
    ) -> Result<Vec<stripe::CreateCheckoutSessionLineItems>, StripeHelperError> {
        use stripe::CreateCheckoutSessionLineItems;

        let plan_product_key = format!("{}-plan", subscription.service_key);
        let mut line_items = Vec::new();

        let plan_product_id = self
            .find_or_register_product(&plan_product_key, subscription.tax_class)
            .await?;
        let plan_price = self
            .plan_price(&plan_product_id, &mut *subscription)
            .await?;
        line_items.push(CreateCheckoutSessionLineItems {
            price: Some(plan_price.id.to_string()),
            quantity: Some(1),
            ..Default::default()
        });

        let bandwidth_product_id = self
            .find_or_register_product(BANDWIDTH_PRODUCT_KEY, subscription.tax_class)
            .await?;
        let bandwidth_price = self
            .bandwidth_price(&bandwidth_product_id, &mut *subscription)
            .await?;
        line_items.push(CreateCheckoutSessionLineItems {
            price: Some(bandwidth_price.id.to_string()),
            ..Default::default()
        });

        let storage_product_id = self
            .find_or_register_product(STORAGE_PRODUCT_KEY, subscription.tax_class)
            .await?;
        let storage_price = self
            .storage_price(&storage_product_id, &mut *subscription)
            .await?;
        line_items.push(CreateCheckoutSessionLineItems {
            price: Some(storage_price.id.to_string()),
            ..Default::default()
        });

        Ok(line_items)
    }

    async fn storage_price(
        &self,
        storage_product_id: &str,
        subscription: &mut Subscription,
    ) -> Result<stripe::Price, StripeHelperError> {
        use stripe::{
            CreatePrice, CreatePriceRecurring, CreatePriceRecurringAggregateUsage,
            CreatePriceRecurringInterval, CreatePriceRecurringUsageType, CreatePriceTiers,
            Currency, IdOrCreate, Metadata, Price, PriceBillingScheme, PriceTaxBehavior,
            PriceTiersMode, UpTo, UpToOther,
        };

        // If we already have a cached price associated with bandwidth pricing, verify its still
        // valid on Stripe then return it, otherwise we'll create a new one.
        if let Some(stripe_price_id) = &subscription.hot_storage_stripe_price_id {
            if let Some(price) = self.find_price_by_id(stripe_price_id).await? {
                return Ok(price);
            }
        }

        let mut params = CreatePrice::new(Currency::USD);

        params.expand = &["product"];
        params.product = Some(IdOrCreate::Id(storage_product_id));

        let price = subscription
            .hot_storage_price
            .as_ref()
            .ok_or(StripeHelperError::MissingPrice)?;

        params.billing_scheme = Some(PriceBillingScheme::Tiered);
        params.tiers_mode = Some(PriceTiersMode::Graduated);
        params.tiers = Some(vec![
            CreatePriceTiers {
                flat_amount: Some(0),
                up_to: Some(UpTo::Max(subscription.included_hot_storage as u64)),
                ..Default::default()
            },
            CreatePriceTiers {
                unit_amount_decimal: Some(price.in_fractional_cents()),
                up_to: Some(UpTo::Other(UpToOther::Inf)),
                ..Default::default()
            },
        ]);

        // Set the nature of our payment (recurring monthly, billed monthly subscription)
        params.recurring = Some(CreatePriceRecurring {
            aggregate_usage: Some(CreatePriceRecurringAggregateUsage::Sum),
            interval: CreatePriceRecurringInterval::Month,
            interval_count: Some(1),
            usage_type: Some(CreatePriceRecurringUsageType::Metered),
            ..Default::default()
        });

        // Tax related settings need to be set as well
        params.tax_behavior = Some(PriceTaxBehavior::Exclusive);

        params.metadata = Some(Metadata::from([(
            METADATA_SUBSCRIPTION_KEY.to_string(),
            subscription.id.to_string(),
        )]));

        let price = Price::create(&self.client, params).await?;

        let mut conn = self.database.acquire().await?;
        subscription
            .persist_storage_price_stripe_id(&mut conn, price.id.as_str())
            .await?;

        Ok(price)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StripeHelperError {
    #[error("failure while querying the database: {0}")]
    DatabaseFailure(#[from] sqlx::Error),

    #[error("attempted to access price without one being available")]
    MissingPrice,

    #[error("failed to located user that should have existed")]
    MissingUser,

    #[error("failure in making a request to the stripe API: {0}")]
    StripeClientError(#[from] stripe::StripeError),

    #[error("error building up stripe ID: {0}")]
    StripeIdError(#[from] stripe::ParseIdError),
}

async fn register_stripe_product(
    client: &stripe::Client,
    product_key: &str,
    tax_class: TaxClass,
    title: &str,
) -> Result<stripe::Product, StripeHelperError> {
    use stripe::{CreateProduct, Metadata, Product, ProductType};

    let descriptor = format!("{}-{}", PRODUCT_DESCRIPTOR_PREFIX, product_key).to_uppercase();
    let metadata = Metadata::from([
        (METADATA_PRODUCT_KEY.to_string(), product_key.to_string()),
        (PRODUCT_TAXCLASS_KEY.to_string(), tax_class.to_string()),
    ]);

    let mut params = CreateProduct::new(title);
    params.shippable = Some(false);
    params.type_ = Some(ProductType::Service);
    params.statement_descriptor = Some(&descriptor);
    params.metadata = Some(metadata);
    params.tax_code = tax_class.stripe_id();

    let new_product = Product::create(client, params).await?;
    Ok(new_product)
}

async fn search_products_for_key(
    client: &stripe::Client,
    product_key: &str,
    tax_class: TaxClass,
) -> Result<Option<String>, StripeHelperError> {
    use stripe::{ListProducts, Product};

    let search_params = ListProducts {
        active: Some(true),
        ..Default::default()
    };

    let existing_products = Product::list(client, &search_params).await?;
    for product in existing_products.data.iter() {
        let metadata = match &product.metadata {
            Some(m) => m,
            None => continue,
        };

        let m_product_key = metadata.get(METADATA_PRODUCT_KEY);
        let m_tax_class_key = metadata.get(PRODUCT_TAXCLASS_KEY);

        match (m_product_key, m_tax_class_key) {
            (Some(key), Some(tax_str)) => {
                let m_tax_class = match TaxClass::try_from(tax_str.as_str()) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                // confirm both the product key and tax type match
                if product_key == key && tax_class == m_tax_class {
                    return Ok(Some(product.id.to_string()));
                }
            }
            _ => continue,
        }
    }

    Ok(None)
}
