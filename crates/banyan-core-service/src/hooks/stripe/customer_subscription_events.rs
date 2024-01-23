use crate::database::DatabaseConnection;
use crate::database::models::{SubscriptionStatus, User};
use crate::hooks::stripe::StripeWebhookError;

pub async fn manage(
    conn: &mut DatabaseConnection,
    subscription: &stripe::Subscription,
) -> Result<(), StripeWebhookError> {
    let stripe_customer_id = subscription.customer.id().to_string();

    let user = User::find_by_stripe_customer_id(&mut *conn, stripe_customer_id)
        .await?
        .ok_or(StripeWebhookError::MissingTarget)?;

    let stripe_subscription_id = subscription.id.to_string();
    let new_subscription_status = SubscriptionStatus::from(subscription.status);

    // need to enable subscription if paid
    // need to disable if unpaid
    todo!()
}
