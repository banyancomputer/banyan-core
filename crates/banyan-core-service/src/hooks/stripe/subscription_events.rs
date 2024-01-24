use time::OffsetDateTime;

use crate::app::stripe_helper::{METADATA_SUBSCRIPTION_KEY, METADATA_USER_KEY};
use crate::database::models::{Subscription, SubscriptionStatus, User};
use crate::database::DatabaseConnection;
use crate::hooks::stripe::StripeWebhookError;

pub async fn handler(
    conn: &mut DatabaseConnection,
    stripe_subscription: &stripe::Subscription,
) -> Result<(), StripeWebhookError> {
    let _stripe_customer_id = stripe_subscription.customer.id().to_string();

    let meta_user_id = stripe_subscription.metadata
        .get(METADATA_USER_KEY)
        .ok_or(StripeWebhookError::MissingData)?;
    let _user = User::find_by_id(&mut *conn, &meta_user_id)
        .await?
        .ok_or(StripeWebhookError::MissingTarget)?;

    let meta_subscription_id = stripe_subscription.metadata
        .get(METADATA_SUBSCRIPTION_KEY)
        .ok_or(StripeWebhookError::MissingData)?;
    let _subscription = Subscription::find_by_id(&mut *conn, &meta_subscription_id)
        .await?
        .ok_or(StripeWebhookError::MissingTarget)?;

    let _stripe_subscription_id = stripe_subscription.id.to_string();
    let _new_subscription_status = SubscriptionStatus::from(stripe_subscription.status);
    let _valid_until = OffsetDateTime::from_unix_timestamp(stripe_subscription.current_period_end)
        .map_err(|_| StripeWebhookError::MissingData)?;

    //sqlx::query!(
    //    r#"UPDATE users
    //         SET stripe_customer_id = $1,
    //             stripe_subscription_id = $2,
    //             subscription_id = $3,
    //             subscription_status = $4,
    //             subscription_valid_until = $5
    //         WHERE id = $6;"#,
    //    stripe_customer_id,
    //    stripe_subscription_id,
    //    subscription.id,
    //    new_subscription_status,
    //    valid_until,
    //    user.id,
    //)
    //.execute(&mut *conn)
    //.await?;

    Ok(())
}
