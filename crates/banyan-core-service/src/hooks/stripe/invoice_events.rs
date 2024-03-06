use time::OffsetDateTime;

use crate::app::stripe_helper::{METADATA_SUBSCRIPTION_KEY, METADATA_USER_KEY};
use crate::database::models::{Invoice, InvoiceStatus, NewInvoice, PriceUnits, Subscription, User};
use crate::database::DatabaseConnection;
use crate::hooks::stripe::StripeWebhookError;

pub async fn creation_handler(
    conn: &mut DatabaseConnection,
    stripe_invoice: &stripe::Invoice,
) -> Result<(), StripeWebhookError> {
    let stripe_invoice_id = stripe_invoice.id.to_string();
    tracing::warn!("create handler for invoice:{stripe_invoice_id}");

    let stripe_customer_id = stripe_invoice
        .customer
        .as_ref()
        .ok_or(StripeWebhookError::missing_data(
            "invoice_create/customer/id",
        ))?
        .id()
        .to_string();

    let total_amount = stripe_invoice
        .amount_due
        .map(PriceUnits::from_cents)
        .ok_or(StripeWebhookError::missing_data(
            "invoice_create/total_amount",
        ))?;
    let invoice_status = stripe_invoice
        .status
        .map(InvoiceStatus::from)
        .ok_or(StripeWebhookError::missing_data("invoice_create/status"))?;

    let period_start = stripe_invoice
        .period_start
        .ok_or(StripeWebhookError::missing_data(
            "invoice_create/period_start",
        ))?;
    let billing_start = OffsetDateTime::from_unix_timestamp(period_start)
        .map_err(|_| StripeWebhookError::invalid_data("invoice_create/billing_start"))?;

    let period_end = stripe_invoice
        .period_end
        .ok_or(StripeWebhookError::missing_data(
            "invoice_create/period_end",
        ))?;
    let billing_end = OffsetDateTime::from_unix_timestamp(period_end)
        .map_err(|_| StripeWebhookError::invalid_data("invoice_create/billing_end"))?;

    let stripe_sub_det = stripe_invoice
        .subscription_details
        .as_ref()
        .ok_or(StripeWebhookError::missing_data("invoice_create/sub_det"))?;

    let stripe_sub_metadata =
        stripe_sub_det
            .metadata
            .as_ref()
            .ok_or(StripeWebhookError::missing_data(
                "invoice_create/sub_det/meta",
            ))?;

    let m_user_id =
        stripe_sub_metadata
            .get(METADATA_USER_KEY)
            .ok_or(StripeWebhookError::missing_data(
                "invoice_create/sub_det/meta/db_user_id",
            ))?;
    let m_subscription_id = stripe_sub_metadata.get(METADATA_SUBSCRIPTION_KEY).ok_or(
        StripeWebhookError::missing_data("invoice_create/sub_det/meta/db_subscription_id"),
    )?;

    NewInvoice {
        user_id: m_user_id,

        stripe_invoice_id: &stripe_invoice_id,
        stripe_customer_id: &stripe_customer_id,

        billing_start: &billing_start,
        billing_end: &billing_end,

        subscription_id: m_subscription_id,

        total_amount,
        status: invoice_status,
    }
    .save(&mut *conn)
    .await?;

    Ok(())
}

pub async fn update_handler(
    conn: &mut DatabaseConnection,
    stripe_invoice: &stripe::Invoice,
) -> Result<(), StripeWebhookError> {
    let stripe_invoice_id = stripe_invoice.id.to_string();
    tracing::warn!("update handler for invoice:{stripe_invoice_id}");

    let new_status = stripe_invoice
        .status
        .map(InvoiceStatus::from)
        .ok_or(StripeWebhookError::missing_data("invoice_update/status"))?;

    let mut invoice = Invoice::from_stripe_invoice_id(&mut *conn, &stripe_invoice_id)
        .await?
        .ok_or(StripeWebhookError::missing_target("db_invoice"))?;
    invoice.update_status(&mut *conn, new_status).await?;

    // Grab the user associated with the invoice
    let mut user = User::by_id(&mut *conn, &invoice.user_id).await?;
    // Grab the user's subscription
    let subscription = Subscription::find_by_id(&mut *conn, &user.subscription_id)
        .await?
        .ok_or(StripeWebhookError::missing_data("db subscription"))?;
    // If the user doesn't already have the included amount, bring them up to it
    let included = subscription.included_archival;
    if user.earned_tokens < included {
        user.award_tokens(conn, included - user.earned_tokens)
            .await?;
    }

    if let Some(maximum) = subscription.archival_hard_limit {
        let monthly_dosage = (maximum - included) / 6;
        // Give the user their montly allotment up to the maximum
        let tokens_earned = std::cmp::min(maximum - user.earned_tokens, monthly_dosage);
        // If the user is not already over their plan capacity / downgraded
        if tokens_earned > 0 {
            user.award_tokens(conn, tokens_earned).await?;
        }
    }

    Ok(())
}
