use std::env;

use lettre::{
    message::Message,
    transport::{
        smtp::{authentication::Credentials, SmtpTransport},
        stub::StubTransport,
    },
    Transport,
};

use crate::error::CoreError;

pub enum EmailTransport {
    Smtp(SmtpTransport),
    Stub(StubTransport),
}

impl EmailTransport {
    pub fn new() -> Result<Self, CoreError> {
        let smtp_url = env::var("SMTP_URL");
        let (host, port, creds) = match smtp_url {
            Ok(smtp_url) => parse_smtp_url(&smtp_url).map_err(|e| {
                CoreError::default_error(&format!("CoreError parsing SMTP_URL: {}", e))
            })?,
            Err(_) => {
                return Ok(EmailTransport::Stub(StubTransport::new_ok()));
            }
        };
        let transport = SmtpTransport::starttls_relay(&host)
            .map_err(|e| {
                CoreError::default_error(&format!("CoreError creating SMTP transport: {}", e))
            })?
            .credentials(creds)
            .port(port);
        Ok(EmailTransport::Smtp(transport.build()))
    }

    pub fn send(&self, message: Message) -> Result<(), CoreError> {
        match self {
            EmailTransport::Smtp(transport) => {
                transport
                    .send(&message)
                    // TODO: What should we be doing with the response here?
                    .map(|_| ()) // Simply discard the Response here for now
                    .map_err(|e| {
                        CoreError::default_error(&format!("CoreError sending email: {}", e))
                    })
            }
            EmailTransport::Stub(transport) => {
                // TODO: What else should be logged here?
                tracing::info!("Outgoing email: {}", std::str::from_utf8(&message.formatted()).map_err(|e| {
                    CoreError::default_error(&format!("CoreError could not decode email: {}", e))
                })?);
                transport.send(&message).map_err(|e| {
                    CoreError::default_error(&format!("CoreError sending email: {}", e))
                })
            }
        }
    }
}


// Parse a url of form
// <host>:<port>?<username>:<password>
// into a tuple of (host, port, creds)
fn parse_smtp_url(url: &str) -> Result<(String, u16, Credentials), CoreError> {
    let mut parts = url.split("?");

    let host_port = parts.next().ok_or(CoreError::default_error("missing host:port"))?;
    let creds = parts.next().ok_or(CoreError::default_error("missing username:password"))?;

    let mut host_parts = host_port.split(":");
    let host = host_parts.next().ok_or(CoreError::default_error("missing host"))?.to_string();
    let port: u16 = host_parts.next().ok_or(CoreError::default_error("missing port"))?.parse().map_err(|e| {
        CoreError::default_error(&format!("Error parsing port: {}", e))
    })?;

    let mut creds_parts = creds.split(":");
    let username = creds_parts.next().ok_or(CoreError::default_error("missing username"))?.to_string();
    let password = creds_parts.next().ok_or(CoreError::default_error("missing password"))?.to_string();

    Ok((host, port, Credentials::new(username, password)))
}


#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn stub_transport() -> Result<(), CoreError> {
        env::remove_var("SMTP_URL");
        let transport = EmailTransport::new()?;
        match transport {
            EmailTransport::Stub(_) => Ok(()),
            _ => Err(CoreError::default_error("Expected StubTransport")),
        }
    }

    #[test]
    #[serial] 
    fn smtp_transport() -> Result<(), CoreError> {
        env::set_var("SMTP_URL", "localhost:1025?username:password");
        let transport = EmailTransport::new()?;
        match transport {
            EmailTransport::Smtp(_) => Ok(()),
            _ => Err(CoreError::default_error("Expected SmtpTransport")),
        }
    }

    #[test]
    #[serial]
    fn invalid_smtp_url() -> Result<(), CoreError> {
        env::set_var("SMTP_URL", "localhost:1025");
        let transport = EmailTransport::new();
        assert!(transport.is_err());
        Ok(())
    }


}
