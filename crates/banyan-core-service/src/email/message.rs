use std::env;

use lazy_static::lazy_static;
use lettre::message::{
    header::{Header, HeaderName, HeaderValue},
    Message,
};
use serde::{ser::StdError, Serialize, Serializer};
use serde_json::Value;

use crate::email::error::EmailError;

use super::template_registry::TemplateRegistry;

// Help Implementing a new email message:

// 1. Add a new template to the src/email/templates directory. The name of the template should be <snake_case_name>.hbs
//     This next line makes it available for our message builder to use here
lazy_static! {
    static ref TEMPLATE_REGISTRY: TemplateRegistry = TemplateRegistry::default();
}

// 2. Add a new variant to the EmailMessage enum with requireed tuple fields. Document the fields
pub enum EmailMessage {
    // No fields required
    GaRelease,
}

impl EmailMessage {
    /// Build an email message variant from the given template and data
    pub fn build(&self, recipient_email: &str) -> Result<Message, EmailError> {
        let test_mode = env::var("MAILGUN_TEST_MODE")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .map_err(EmailError::invalid_test_mode)?;

        let mut builder = Message::builder();

        if test_mode {
            builder = builder.clone().header(MailgunTestMode);
        }

        builder
            .from(
                env::var("SMTP_FROM")
                    .map_err(|_| EmailError::missing_smtp_from())?
                    .parse()
                    .map_err(EmailError::invalid_smtp_from)?,
            )
            .to(recipient_email
                .parse()
                .map_err(EmailError::invalid_smtp_from)?)
            .subject(self.subject())
            .body(self.body()?)
            .map_err(EmailError::message_build_error)
    }

    // 3. Implement the subject for the new variant
    fn subject(&self) -> String {
        match self {
            EmailMessage::GaRelease => "Accouncing Banyan GA".to_string(),
        }
    }

    // 4. Implement the body for the new variant -- remember that template you added in step 1? Use it here!
    //     You should be able to access it with its <snake_case_name>
    fn body(&self) -> Result<String, EmailError> {
        match self {
            EmailMessage::GaRelease => TEMPLATE_REGISTRY.render("ga_release", &self),
        }
    }
}

// 5. Implement Serialize for the new variant. This should take the tuple fields and serialize them into a JSON object
//   that can be passed to the template.
impl Serialize for EmailMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            EmailMessage::GaRelease => {
                // No fields required, return an empty object
                let map = Value::Object(serde_json::Map::new());
                map.serialize(serializer)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const RECIPIENT: &str = "fake@email.com";

    // 6. Add a test for the new variant
    #[test]
    fn ga_release() -> Result<(), EmailError> {
        let _message = EmailMessage::GaRelease.build(RECIPIENT)?;
        Ok(())
    }

    // Mailgun Test Mode Switch
    mod test_mode {
        use super::*;
        use serial_test::serial;

        #[test]
        #[serial]
        fn clean_env() -> Result<(), EmailError> {
            env::set_var("SMTP_FROM", RECIPIENT);
            env::remove_var("MAILGUN_TEST_MODE");
            let message = EmailMessage::GaRelease.build(RECIPIENT)?;
            assert!(message.headers().get::<MailgunTestMode>().is_none());
            Ok(())
        }

        #[test]
        #[serial]
        fn true_env() -> Result<(), EmailError> {
            env::set_var("MAILGUN_TEST_MODE", "true");
            let message = EmailMessage::GaRelease.build(RECIPIENT)?;
            assert!(message.headers().get::<MailgunTestMode>().is_some());
            Ok(())
        }

        #[test]
        #[serial]
        fn false_env() -> Result<(), EmailError> {
            env::set_var("MAILGUN_TEST_MODE", "false");
            let message = EmailMessage::GaRelease.build(RECIPIENT)?;
            assert!(message.headers().get::<MailgunTestMode>().is_none());
            Ok(())
        }

        #[test]
        #[serial]
        fn invalid_env() -> Result<(), EmailError> {
            env::set_var("MAILGUN_TEST_MODE", "invalid");
            let message = EmailMessage::GaRelease.build(RECIPIENT);
            assert!(message.is_err());
            Ok(())
        }
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
