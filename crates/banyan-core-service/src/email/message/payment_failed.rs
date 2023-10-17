use serde::{Deserialize, Serialize};

use super::EmailMessage;

#[derive(Serialize, Deserialize)]
pub struct PaymentFailed;

impl EmailMessage for PaymentFailed {
    const SUBJECT: &'static str = "Payment Failed";
    const TEMPLATE_NAME: &'static str = "payment_failed";
    const TYPE_NAME: &'static str = "payment_failed";
}
