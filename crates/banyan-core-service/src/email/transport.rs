use lettre::{
    message::Message,
    transport::{smtp::SmtpTransport, stub::StubTransport},
    Transport,
};

use super::{config::SmtpConnection, error::EmailError};

pub enum EmailTransport {
    Smtp(SmtpTransport),
    Stub(StubTransport),
}

impl EmailTransport {
    pub fn new(maybe_smtp_connection: Option<&SmtpConnection>) -> Result<Self, EmailError> {
        match maybe_smtp_connection {
            Some(smtp_connection) => Ok(EmailTransport::Smtp(
                SmtpTransport::starttls_relay(smtp_connection.host())
                    .map_err(EmailError::smtp_transport_build_error)?
                    .credentials(smtp_connection.creds().clone())
                    .port(smtp_connection.port())
                    .build(),
            )),
            None => {
                // Use the StubTransport if no SMTP_URL is provided
                Ok(EmailTransport::Stub(StubTransport::new_ok()))
            }
        }
    }

    pub fn send(&self, message: &Message) -> Result<(), EmailError> {
        match self {
            EmailTransport::Smtp(transport) => {
                transport
                    .send(&message)
                    // TODO: What should we be doing with the response here?
                    .map(|_| ()) // Simply discard the Response here for now
                    .map_err(EmailError::smtp_send_error)
            }
            EmailTransport::Stub(transport) => {
                // TODO: What else should be logged here?
                tracing::info!(
                    "Outgoing email: {}",
                    std::str::from_utf8(&message.formatted()).map_err(EmailError::utf8_error)?
                );
                transport
                    .send(&message)
                    .map_err(EmailError::stub_send_error)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_transport() -> Result<(), EmailError> {
        let transport = EmailTransport::new(None)?;
        match transport {
            EmailTransport::Stub(_) => Ok(()),
            _ => Err(EmailError::default_error("Expected StubTransport")),
        }
    }

    #[test]
    fn smtp_transport() -> Result<(), EmailError> {
        let transport = EmailTransport::new(Some(&SmtpConnection::new("smtps://user:pass@host")?))?;
        match transport {
            EmailTransport::Smtp(_) => Ok(()),
            _ => Err(EmailError::default_error("Expected SmtpTransport")),
        }
    }
}
