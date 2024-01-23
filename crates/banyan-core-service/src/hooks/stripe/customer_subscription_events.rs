use time::OffsetDateTime;

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
    let valid_until = OffsetDateTime::from_unix_timestamp(subscription.current_period_end)
        .map_err(|_| StripeWebhookError::MissingData)?;

    if user.active_stripe_subscription_id.as_ref() == Some(&stripe_subscription_id) {
        if user.active_subscription_status == new_subscription_status {
            // Nothing to be done
            return Ok(());
        }

        sqlx::query!(
            r#"UPDATE users
                 SET active_subscription_status = $1,
                     active_subscription_valid_until = $2
                 WHERE id = $3;"#,
            new_subscription_status,
            valid_until,
            user.id,
        )
        .execute(&mut *conn)
        .await?;

        return Ok(());
    }

    if user.pending_stripe_subscription_id.as_ref() == Some(&stripe_subscription_id) {
        if new_subscription_status == SubscriptionStatus::Active {
            // Becoming active replaces the active one
            sqlx::query!(
                r#"UPDATE users
                     SET active_stripe_subscription_id = pending_stripe_subscription_id,
                         active_subscription_id = pending_subscription_id,
                         active_subscription_status = $1,
                         active_subscription_valid_until = $2,
                         pending_stripe_subscription_id = NULL,
                         pending_subscription_id = NULL,
                         pending_subscription_expiration = NULL
                     WHERE id = $3;"#,
                new_subscription_status,
                valid_until,
                user.id,
            )
            .execute(&mut *conn)
            .await?;
        }

        return Ok(());
    }

    Err(StripeWebhookError::MissingTarget)
}
