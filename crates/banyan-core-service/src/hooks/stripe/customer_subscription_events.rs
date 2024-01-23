use crate::database::DatabaseConnection;
use crate::hooks::stripe::StripeWebhookError;

pub async fn manage(
    conn: &mut DatabaseConnection,
    subscription: &stripe::Subscription,
) -> Result<(), StripeWebhookError> {
    // need to enable subscription if paid
    // need to disable if unpaid
    todo!()
}
