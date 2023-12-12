use lazy_static::lazy_static;
use lettre::message::header::{Header, HeaderName, HeaderValue};
use lettre::message::Message;
use serde::de::DeserializeOwned;
use serde::ser::StdError;
use serde::Serialize;
use uuid::Uuid;

use super::error::EmailError;
use super::template_registry::TemplateRegistry;
use super::transport::EmailTransport;

// Help Adding a new email content variant:

// 1. Add a new handlebars template to the ./templates/email directory. The name of the template should be <snake_case_name>.hbs
//     This next line makes it available for our message builder to use here
lazy_static! {
    static ref TEMPLATE_REGISTRY: TemplateRegistry = TemplateRegistry::default();
}

// 2. Create a new module in `message` for your new email message.
//     In your module define a struct that contains the templated data for your new email message.
//      Impl Email Message for your templated data. See message/ga_release.rs for an example.
mod ga_release;
mod payment_failed;
mod product_invoice;
mod reaching_storage_limit;
mod scheduled_maintenance;

pub use ga_release::GaRelease;
pub use payment_failed::PaymentFailed;
pub use product_invoice::ProductInvoice;
pub use reaching_storage_limit::ReachingStorageLimit;
pub use scheduled_maintenance::ScheduledMaintenance;

pub trait EmailMessage:
    Serialize + DeserializeOwned + Sized + std::marker::Send + std::marker::Sync + 'static
{
    const SUBJECT: &'static str;
    const TEMPLATE_NAME: &'static str;
    const TYPE_NAME: &'static str;

    fn send(
        &self,
        transport: &EmailTransport,
        from: &str,
        to: &str,
        message_id: Uuid,
        test_mode: bool,
    ) -> Result<(), EmailError> {
        let message = self.build(from, to, message_id, test_mode)?;
        transport.send(&message)?;
        Ok(())
    }

    fn build(
        &self,
        from: &str,
        to: &str,
        message_id: Uuid,
        test_mode: bool,
    ) -> Result<Message, EmailError> {
        let mut builder = Message::builder();
        if test_mode {
            builder = builder.header(MailgunTestMode);
        }
        builder
            .header(MailgunMessageId(message_id))
            .from(from.parse().map_err(EmailError::invalid_from_address)?)
            .to(to.parse().map_err(EmailError::invalid_to_address)?)
            .subject(Self::SUBJECT)
            .body(TEMPLATE_REGISTRY.render(Self::TEMPLATE_NAME, self)?)
            .map_err(EmailError::message_build_error)
    }

    fn type_name(&self) -> &'static str {
        Self::TYPE_NAME
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    lazy_static! {
        static ref TRANSPORT: EmailTransport = EmailTransport::new(None).unwrap();
    }
    const MESSAGE_ID: Uuid = Uuid::nil();
    const FROM: &str = "fake@email.com";
    const TO: &str = "another_fake@email.com";

    // 3. Add a test for the new variant in order to make sure it builds correctly.

    #[test]
    fn ga_release_send() -> Result<(), EmailError> {
        GaRelease.send(&TRANSPORT, FROM, TO, MESSAGE_ID, false)?;
        Ok(())
    }

    #[test]
    fn payment_failed_send() -> Result<(), EmailError> {
        PaymentFailed.send(&TRANSPORT, FROM, TO, MESSAGE_ID, false)?;
        Ok(())
    }

    #[test]
    fn product_invoice_send() -> Result<(), EmailError> {
        ProductInvoice {
            url: "https://www.banyansecurity.io".parse().unwrap(),
        }
        .send(&TRANSPORT, FROM, TO, MESSAGE_ID, false)?;
        Ok(())
    }

    #[test]
    fn reaching_storage_limit_send() -> Result<(), EmailError> {
        ReachingStorageLimit {
            current_usage: 10,
            max_usage: 11,
        }
        .send(&TRANSPORT, FROM, TO, MESSAGE_ID, false)?;
        Ok(())
    }

    #[test]
    fn scheduled_maintenance_send() -> Result<(), EmailError> {
        ScheduledMaintenance {
            start: "2020-01-01".to_string(),
            end: "2020-01-02".to_string(),
        }
        .send(&TRANSPORT, FROM, TO, MESSAGE_ID, false)?;
        Ok(())
    }

    // Mailgun Test Mode Switch Tests

    #[test]
    fn test_mode_true() -> Result<(), EmailError> {
        let message = GaRelease.build(FROM, TO, MESSAGE_ID, true)?;
        assert!(message.headers().get::<MailgunTestMode>().is_some());
        Ok(())
    }

    #[test]
    fn test_mode_false() -> Result<(), EmailError> {
        let message = GaRelease.build(FROM, TO, MESSAGE_ID, false)?;
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

#[derive(Clone, Copy, Debug)]
struct MailgunMessageId(Uuid);

impl Header for MailgunMessageId {
    fn name() -> HeaderName {
        HeaderName::new_from_ascii_str("X-Mailgun-Variables")
    }
    fn parse(s: &str) -> Result<Self, Box<dyn StdError + Send + Sync>> {
        let json: serde_json::Value = serde_json::from_str(s)?;
        let message_id = json
            .get("message_id")
            .ok_or("missing message_id")?
            .as_str()
            .ok_or("message_id is not a string")?;
        Ok(MailgunMessageId(Uuid::parse_str(message_id)?))
    }
    fn display(&self) -> HeaderValue {
        let mut map = serde_json::Map::new();
        map.insert(
            "message_id".to_string(),
            serde_json::Value::String(self.0.to_string()),
        );
        HeaderValue::new(
            MailgunMessageId::name(),
            serde_json::to_string(&map).unwrap(),
        )
    }
}
