use serde::{Deserialize, Serialize};
use url::Url;

use super::EmailMessage;

#[derive(Serialize, Deserialize)]
pub struct ProductInvoice {
    /// Where a user can go to view their invoice
    pub(crate) url: Url,
}

impl EmailMessage for ProductInvoice {
    const SUBJECT: &'static str = "Your Banyan Invoice";
    const TEMPLATE_NAME: &'static str = "product_invoice";
    const TYPE_NAME: &'static str = "product_invoice";
}
