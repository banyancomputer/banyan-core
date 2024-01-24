use crate::database::DatabaseConnection;
use crate::hooks::stripe::StripeWebhookError;

pub async fn completed(
    _conn: &mut DatabaseConnection,
    session: &stripe::CheckoutSession,
) -> Result<(), StripeWebhookError> {
    let _session_str = session.id.to_string();

    Ok(())
}

pub async fn expired(
    _conn: &mut DatabaseConnection,
    session: &stripe::CheckoutSession,
) -> Result<(), StripeWebhookError> {
    let _session_str = session.id.to_string();

    Ok(())
}
