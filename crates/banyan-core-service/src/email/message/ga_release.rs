use serde::Serialize;

use super::EmailMessage;

// 1. Create a struct that contains the templated data for your new email message.
#[derive(Serialize)]
pub struct GaRelease;

// 2. Impl Email Message for your templated data
impl EmailMessage for GaRelease {
    // 2a. Set the subject of your email
    const SUBJECT: &'static str = "Announcing Banyan GA Release";
    // 2b. Set the name of your template
    const TEMPLATE_NAME: &'static str = "ga_release";
    // 2c. Set the name of your email message type. This should be unique, and not change!
    const TYPE_NAME: &'static str = "ga_release";
}
