use time::OffsetDateTime;

use crate::database::models::{SubscriptionStatus, User};
use crate::database::DatabaseConnection;
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
    let valid_until = OffsetDateTime::from_unix_timestamp(subscription.current_period_end)
        .map_err(|_| StripeWebhookError::MissingData)?;

    // todo: need to associate to our subscription ID

    sqlx::query!(
        r#"UPDATE users
             SET stripe_subscription_id = $1,
                 subscription_status = $2,
                 subscription_valid_until = $3
             WHERE id = $4;"#,
        stripe_subscription_id,
        new_subscription_status,
        valid_until,
        user.id,
    )
    .execute(&mut *conn)
    .await?;

    Ok(())
}
