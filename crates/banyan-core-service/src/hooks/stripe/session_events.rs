use crate::app::stripe_helper::{METADATA_SUBSCRIPTION_KEY, METADATA_USER_KEY};
use crate::database::models::{
    Invoice, StripeCheckoutSession, Subscription, SubscriptionStatus, User,
};
use crate::database::DatabaseConnection;
use crate::hooks::stripe::StripeWebhookError;

pub async fn handler(
    conn: &mut DatabaseConnection,
    stripe_session: &stripe::CheckoutSession,
) -> Result<(), StripeWebhookError> {
    let stripe_session_str = stripe_session.id.to_string();

    let stripe_metadata = stripe_session
        .metadata
        .as_ref()
        .ok_or(StripeWebhookError::missing_data("session/meta"))?;
    let m_user_id = stripe_metadata
        .get(METADATA_USER_KEY)
        .ok_or(StripeWebhookError::missing_data("session/meta/db_user_id"))?;

    let mut checkout_session =
        StripeCheckoutSession::find_by_stripe_id(&mut *conn, m_user_id, &stripe_session_str)
            .await?
            .ok_or(StripeWebhookError::missing_target("db_checkout_session"))?;
    checkout_session.complete(&mut *conn).await?;

    let user = User::find_by_id(&mut *conn, &checkout_session.user_id)
        .await?
        .ok_or(StripeWebhookError::missing_target("db_user"))?;

    // todo(sstelfox): We need to handle the case where we a user is currently on a subscription,
    // and are migrating to a new subscription.

    let m_subscription_id =
        stripe_metadata
            .get(METADATA_SUBSCRIPTION_KEY)
            .ok_or(StripeWebhookError::missing_data(
                "session/meta/db_subscription_id",
            ))?;
    let subscription = Subscription::find_by_id(&mut *conn, m_subscription_id)
        .await?
        .ok_or(StripeWebhookError::missing_target("db_subscription"))?;

    let stripe_subscription_id = stripe_session
        .subscription
        .as_ref()
        .ok_or(StripeWebhookError::missing_data("session/subscription/id"))?
        .id()
        .to_string();

    let stripe_invoice_id = stripe_session
        .invoice
        .as_ref()
        .ok_or(StripeWebhookError::missing_data("session/invoice/id"))?
        .id();
    let invoice = Invoice::from_stripe_invoice_id(&mut *conn, &stripe_invoice_id)
        .await?
        .ok_or(StripeWebhookError::missing_target("db/invoice"))?;

    sqlx::query!(
        r#"UPDATE users
             SET stripe_subscription_id = $1,
                 subscription_id = $2,
                 subscription_status = $3,
                 subscription_valid_until = $4
             WHERE id = $5;"#,
        stripe_subscription_id,
        subscription.id,
        SubscriptionStatus::Active,
        invoice.billing_end,
        user.id,
    )
    .execute(&mut *conn)
    .await?;

    Ok(())
}
