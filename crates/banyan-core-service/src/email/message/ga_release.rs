use serde::Serialize;

use super::EmailMessage;

// 1. Create a struct that contains the templated data for your new email message.
#[derive(Serialize)]
pub struct GaRelease;

// 2. Impl Email Message for your templated data
impl EmailMessage for GaRelease {
    // 2a. Implement the subject() method for your new struct -- this is the subject line of the email
    fn subject() -> String {
        "Announcing Banyan GA Release".to_string()
    }

    // 2b. Implement the template_name() method for your new struct -- this is the name of the template file within the registry
    fn template_name() -> &'static str {
        "ga_release"
    }
}
