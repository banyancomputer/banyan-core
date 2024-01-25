use crate::app::stripe_helper::{METADATA_SUBSCRIPTION_KEY, METADATA_USER_KEY};
use crate::database::models::{StripeCheckoutSession, Subscription, User};
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
        .ok_or(StripeWebhookError::MissingData)?;
    let m_user_id = stripe_metadata
        .get(METADATA_USER_KEY)
        .ok_or(StripeWebhookError::MissingData)?;

    let mut checkout_session =
        StripeCheckoutSession::find_by_stripe_id(&mut *conn, &m_user_id, &stripe_session_str)
            .await?
            .ok_or(StripeWebhookError::MissingTarget)?;
    checkout_session.complete(&mut *conn).await?;

    let m_subscription_id = stripe_metadata
        .get(METADATA_SUBSCRIPTION_KEY)
        .ok_or(StripeWebhookError::MissingData)?;

    let _stripe_subscription = stripe_session
        .subscription
        .as_ref()
        .ok_or(StripeWebhookError::MissingData)?;
    let _stripe_invoice = stripe_session
        .invoice
        .as_ref()
        .ok_or(StripeWebhookError::MissingData)?;

    let _user = User::find_by_id(&mut *conn, &checkout_session.user_id)
        .await?
        .ok_or(StripeWebhookError::MissingTarget)?;
    let _subscription = Subscription::find_by_id(&mut *conn, &m_subscription_id)
        .await?
        .ok_or(StripeWebhookError::MissingTarget)?;

    // confirm invoice has a payment_status of Paid
    // find or create then update invoice
    // assign subscription, stripe subscription, and valid_until on the user

    Ok(())
}
