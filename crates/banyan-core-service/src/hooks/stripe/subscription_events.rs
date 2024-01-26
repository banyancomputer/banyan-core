use crate::app::stripe_helper::{METADATA_SUBSCRIPTION_KEY, METADATA_USER_KEY};
use crate::database::models::{Subscription, SubscriptionStatus, User};
use crate::database::DatabaseConnection;
use crate::hooks::stripe::StripeWebhookError;

pub async fn deleted(
    conn: &mut DatabaseConnection,
    stripe_subscription: &stripe::Subscription,
) -> Result<(), StripeWebhookError> {
    let subscription_status = SubscriptionStatus::from(stripe_subscription.status);

    // This hook only handles account cancellation
    if subscription_status != SubscriptionStatus::Canceled {
        return Ok(());
    }

    let meta_user_id = stripe_subscription.metadata.get(METADATA_USER_KEY).ok_or(
        StripeWebhookError::missing_data("subscription/meta/db_user_id"),
    )?;
    let user = User::find_by_id(&mut *conn, meta_user_id)
        .await?
        .ok_or(StripeWebhookError::missing_target("db_user"))?;

    let meta_subscription_id = stripe_subscription
        .metadata
        .get(METADATA_SUBSCRIPTION_KEY)
        .ok_or(StripeWebhookError::missing_data(
            "subscription/meta/db_subscription_id",
        ))?;

    // Confirm that the subscription being cancelled is the active one on the user's account, this
    // may occur if stripes cancellation webhook comes in after we've switched them to a new plan.
    if &user.subscription_id != meta_subscription_id {
        tracing::error!("received canceled subscription webhook for unassociated subscription");
        return Ok(());
    }

    let default_subscription_id = Subscription::default_subscription_id(&mut *conn).await?;

    sqlx::query!(
        r#"UPDATE users
             SET subscription_id = $1,
                 subscription_status = $2,
                 subscription_valid_until = NULL
             WHERE id = $3;"#,
        default_subscription_id,
        SubscriptionStatus::Active,
        user.id,
    )
    .execute(&mut *conn)
    .await?;

    Ok(())
}
