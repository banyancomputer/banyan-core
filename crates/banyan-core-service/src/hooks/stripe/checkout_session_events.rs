use time::OffsetDateTime;

use crate::database::models::{StripeCheckoutSession, StripeCheckoutSessionStatus};
use crate::database::DatabaseConnection;
use crate::hooks::stripe::StripeWebhookError;

pub async fn completed(
    conn: &mut DatabaseConnection,
    session: &stripe::CheckoutSession,
) -> Result<(), StripeWebhookError> {
    let session_str = session.id.to_string();
    let session = match StripeCheckoutSession::find_by_id(&mut *conn, &session_str).await? {
        Some(sess) => sess,
        None => return Err(StripeWebhookError::MissingTarget),
    };

    let now = OffsetDateTime::now_utc();
    sqlx::query!(
        "UPDATE stripe_checkout_sessions SET status = $1, completed_at = $2 WHERE status = $3 AND id = $4;",
        StripeCheckoutSessionStatus::Completed,
        now,
        StripeCheckoutSessionStatus::Created,
        session.id,
    )
    .execute(&mut *conn)
    .await?;

    Ok(())
}

pub async fn expired(
    conn: &mut DatabaseConnection,
    session: &stripe::CheckoutSession,
) -> Result<(), StripeWebhookError> {
    let session_str = session.id.to_string();
    let session = match StripeCheckoutSession::find_by_id(&mut *conn, &session_str).await? {
        Some(sess) => sess,
        None => return Err(StripeWebhookError::MissingTarget),
    };

    let now = OffsetDateTime::now_utc();
    sqlx::query!(
        "UPDATE stripe_checkout_sessions SET status = $1, completed_at = $2 WHERE status = $3 AND id = $4;",
        StripeCheckoutSessionStatus::Expired,
        now,
        StripeCheckoutSessionStatus::Created,
        session.id,
    )
    .execute(&mut *conn)
    .await?;

    Ok(())
}
