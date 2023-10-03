use std::env;

use lettre::transport::smtp::authentication::Credentials;

use super::error::EmailError;

pub struct EmailConfig {
    pub(crate) smtp_connection: Option<SmtpConnection>,
    pub(crate) from: String,
}

impl EmailConfig {
    pub fn new(maybe_smtp_url: Option<&str>, from: &str) -> Result<Self, EmailError> {
        let smtp_connection = match maybe_smtp_url {
            Some(smtp_url) => Some(SmtpConnection::new(smtp_url)?),
            None => None,
        };
        Ok(Self {
            smtp_connection,
            from: from.to_string(),
        })
    }

    pub fn from_env() -> Result<Self, EmailError> {
        let smtp_url = env::var("SMTP_URL").ok();

        // Check if the SMTP_URL is an empty string if set
        let smtp_url = if let Some(smtp_url) = &smtp_url {
            if smtp_url.is_empty() {
                None
            } else {
                Some(smtp_url.as_str())
            }
        } else {
            None
        };

        let from = env::var("SMTP_FROM").map_err(|_| EmailError::missing_smtp_from())?;
        Self::new(smtp_url, &from)
    }
}

pub struct SmtpConnection {
    host: String,
    port: u16,
    creds: Credentials,
}

impl SmtpConnection {
    /// Parse a url of form:
    ///     <host>:<port>?<username>:<password>
    /// Into a SmtpConnection
    pub fn new(smtp_url: &str) -> Result<Self, EmailError> {
        let mut parts = smtp_url.split('?');
        let host_port = parts
            .next()
            .ok_or(EmailError::invalid_smtp_url("missing host:port"))?;
        let creds = parts
            .next()
            .ok_or(EmailError::invalid_smtp_url("missing username:password"))?;

        let mut host_parts = host_port.split(':');
        let host = host_parts
            .next()
            .ok_or(EmailError::invalid_smtp_url("missing host"))?
            .to_string();
        // If this is an empty string, then throw an error
        if host.is_empty() {
            return Err(EmailError::invalid_smtp_url("missing host"));
        }

        let port: u16 = host_parts
            .next()
            .ok_or(EmailError::invalid_smtp_url("missing port"))?
            .parse()
            .map_err(|e| EmailError::invalid_smtp_url(&format!("Error parsing port: {}", e)))?;

        let mut creds_parts = creds.split(':');
        let username = creds_parts
            .next()
            .ok_or(EmailError::invalid_smtp_url("missing username"))?
            .to_string();
        // If this is an empty string, then throw an error
        if username.is_empty() {
            return Err(EmailError::invalid_smtp_url("missing username"));
        }

        let password = creds_parts
            .next()
            .ok_or(EmailError::invalid_smtp_url("missing password"))?
            .to_string();
        // If this is an empty string, then throw an error
        if password.is_empty() {
            return Err(EmailError::invalid_smtp_url("missing password"));
        }

        Ok(Self {
            host,
            port,
            creds: Credentials::new(username, password),
        })
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn creds(&self) -> &Credentials {
        &self.creds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_config_from_env() -> Result<(), EmailError> {
        env::set_var("SMTP_URL", "localhost:1025?username:password");
        env::set_var("SMTP_FROM", "fake@email.com");
        let email_config = EmailConfig::from_env()?;
        assert_eq!(email_config.from, "fake@email.com");
        assert!(email_config.smtp_connection.is_some());

        // Now empty
        env::set_var("SMTP_URL", "");
        let email_config = EmailConfig::from_env()?;
        assert!(email_config.smtp_connection.is_none());

        // Now missing
        env::remove_var("SMTP_URL");
        let email_config = EmailConfig::from_env()?;
        assert!(email_config.smtp_connection.is_none());
        Ok(())
    }

    #[test]
    fn smtp_connection() -> Result<(), EmailError> {
        let smtp_url = "localhost:1025?username:password";
        let smtp_connection = SmtpConnection::new(smtp_url)?;
        assert_eq!(smtp_connection.host(), "localhost");
        assert_eq!(smtp_connection.port(), 1025);
        Ok(())
    }

    #[test]
    fn smtp_connection_missing_host() {
        let smtp_url = ":1025?username:password";
        let smtp_connection = SmtpConnection::new(smtp_url);
        assert!(smtp_connection.is_err());
    }

    #[test]
    fn smtp_connection_missing_port() {
        let smtp_url = "localhost:?username:password";
        let smtp_connection = SmtpConnection::new(smtp_url);
        assert!(smtp_connection.is_err());
    }

    #[test]
    fn smtp_connection_missing_username() {
        let smtp_url = "localhost:1025?:password";
        let smtp_connection = SmtpConnection::new(smtp_url);
        assert!(smtp_connection.is_err());
    }

    #[test]
    fn smtp_connection_missing_password() {
        let smtp_url = "localhost:1025?username:";
        let smtp_connection = SmtpConnection::new(smtp_url);
        assert!(smtp_connection.is_err());
    }
}
