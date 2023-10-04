use serde::Serialize;
use url::Url;

use super::EmailMessage;

#[derive(Serialize)]
pub struct ProductInvoice {
    /// Where a user can go to view their invoice
    pub(crate) url: Url
}

impl EmailMessage for ProductInvoice {
    fn subject() -> String {
        "Your Invoice is Ready".to_string()
    }

    fn template_name() -> &'static str {
        "product_invoice"
    }
}
