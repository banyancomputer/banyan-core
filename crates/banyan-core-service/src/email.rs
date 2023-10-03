#![allow(dead_code)]
pub mod message;
mod template_registry;
pub mod transport;

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use crate::email::message::EmailMessage;
    use crate::email::transport::EmailTransport;
    use crate::error::CoreError;

    // You will need to allow your mailgun domain to send to the recipient
    const RECIPIENT: &str = "alex@banyan.computer";

    // Ignore this test by default, since their envs might conflict with other tests in the Email module

    // Requires a valid Mailgun server to be running and configured through the SMTP_URL env var
    #[test]
    #[serial]
    #[ignore]
    fn send_smtp() -> Result<(), CoreError> {
        // Use Test Mode so that the email isn't actually sent
        std::env::set_var("MAILGUN_TEST_MODE", "true");
        let transport = EmailTransport::new()?;
        let message = EmailMessage::GaRelease.build(RECIPIENT)?;
        transport.send(message)?;
        Ok(())
    }

    #[test]
    #[serial]
    #[ignore]
    fn send_stub() -> Result<(), CoreError> {
        // Remove the SMTP_URL env var so that the StubTransport is used
        std::env::remove_var("SMTP_URL");
        let transport = EmailTransport::new()?;
        let message = EmailMessage::GaRelease.build(RECIPIENT)?;
        transport.send(message)?;
        Ok(())
    }
}
