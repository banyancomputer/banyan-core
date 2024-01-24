use crate::database::DatabaseConnection;
use crate::hooks::stripe::StripeWebhookError;

pub async fn handler(
    conn: &mut DatabaseConnection,
    session: &stripe::CheckoutSession,
) -> Result<(), StripeWebhookError> {
    let _session_str = session.id.to_string();

    Ok(())
}
