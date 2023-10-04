use serde::Serialize;

use super::EmailMessage;

#[derive(Serialize)]
pub struct PaymentFailed;

impl EmailMessage for PaymentFailed {
    fn subject() -> String {
        "Payment Failed".to_string()
    }

    fn template_name() -> &'static str {
        "payment_failed"
    }
}