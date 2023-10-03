use lazy_static::lazy_static;
use lettre::message::{
    header::{Header, HeaderName, HeaderValue},
    Message,
};
use serde::{ser::StdError, Serialize};

use crate::email::error::EmailError;

use super::template_registry::TemplateRegistry;

// Help Implementing a new email message:

// 1. Add a new template to the ./templates/email directory. The name of the template should be <snake_case_name>.hbs
//     This next line makes it available for our message builder to use here
lazy_static! {
    static ref TEMPLATE_REGISTRY: TemplateRegistry = TemplateRegistry::default();
}

// 2. Create a struct that contains the templated data for your new email message. Impl EmailMessage for it.
#[derive(Serialize)]
pub struct GaRelease;

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

pub trait EmailMessage: Serialize + Sized {
    /// Build an email message variant from the given template and data
    fn build(&self, from: &str, to: &str, test_mode: bool) -> Result<Message, EmailError> {
        let mut builder = Message::builder();
        if test_mode {
            builder = builder.header(MailgunTestMode);
        }
        builder
            .from(from.parse().map_err(EmailError::invalid_from_address)?)
            .to(to.parse().map_err(EmailError::invalid_to_address)?)
            .subject(Self::subject())
            .body(TEMPLATE_REGISTRY.render(Self::template_name(), self)?)
            .map_err(EmailError::message_build_error)
    }

    fn subject() -> String;
    fn template_name() -> &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;

    const FROM: &str = "fake@email.com";
    const TO: &str = "another_fake@email.com";

    // 6. Add a test for the new variant in order to make sure it builds correctly. Make sure it is serial!

    #[test]
    fn ga_release() -> Result<(), EmailError> {
        let _message = GaRelease.build(FROM, TO, false)?;
        Ok(())
    }

    // Mailgun Test Mode Switch Tests

    #[test]
    fn test_mode_true() -> Result<(), EmailError> {
        let message = GaRelease.build(FROM, TO, true)?;
        assert!(message.headers().get::<MailgunTestMode>().is_some());
        Ok(())
    }

    #[test]
    fn test_mode_false() -> Result<(), EmailError> {
        let message = GaRelease.build(FROM, TO, false)?;
        assert!(message.headers().get::<MailgunTestMode>().is_none());
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
struct MailgunTestMode;

impl Header for MailgunTestMode {
    fn name() -> HeaderName {
        HeaderName::new_from_ascii_str("X-Mailgun-Drop-Message")
    }
    fn parse(s: &str) -> Result<Self, Box<dyn StdError + Send + Sync>> {
        if s == "yes" {
            Ok(MailgunTestMode)
        } else {
            Err("invalid value".into())
        }
    }
    fn display(&self) -> HeaderValue {
        HeaderValue::new(MailgunTestMode::name(), "yes".to_string())
    }
}
