use std::env;

use lettre::transport::smtp::authentication::Credentials;
use url::Url;

use super::error::EmailError;
use super::transport::EmailTransport;

#[derive(Clone)]
pub struct EmailConfig {
    smtp_connection: Option<SmtpConnection>,
    from: String,
    test_mode: bool,
}

impl EmailConfig {
    pub fn new(
        maybe_smtp_url: Option<&str>,
        from: &str,
        test_mode: bool,
    ) -> Result<Self, EmailError> {
        let smtp_connection = match maybe_smtp_url {
            Some(smtp_url) => Some(SmtpConnection::new(smtp_url)?),
            None => None,
        };
        Ok(Self {
            smtp_connection,
            from: from.to_string(),
            test_mode,
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

        let test_mode = env::var("MAILGUN_TEST_MODE")
            .map(|v| v == "true")
            .unwrap_or(false);

        Self::new(smtp_url, &from, test_mode)
    }

    pub fn transport(&self) -> Result<EmailTransport, EmailError> {
        EmailTransport::new(self.smtp_connection())
    }

    pub fn smtp_connection(&self) -> Option<&SmtpConnection> {
        self.smtp_connection.as_ref()
    }

    pub fn from(&self) -> &str {
        &self.from
    }

    pub fn test_mode(&self) -> bool {
        self.test_mode
    }
}

#[derive(Clone)]
pub struct SmtpConnection {
    host: String,
    port: u16,
    username: String,
    password: String,
}

impl SmtpConnection {
    pub fn new(smtp_url: &str) -> Result<Self, EmailError> {
        let url = Url::parse(smtp_url)
            .map_err(|e| EmailError::invalid_smtp_url(&format!("Invalid SMTP URL: {}", e)))?;
        if url.scheme() != "smtps" {
            return Err(EmailError::invalid_smtp_url(
                "SMTP URL must use the smtps scheme",
            ));
        };
        let username = url.username();
        if username.is_empty() {
            return Err(EmailError::invalid_smtp_url(
                "SMTP URL must contain a username",
            ));
        };

        // If the username would have had an @ in it, correct it
        let username = username.replace("%40", "@");

        let password = url
            .password()
            .ok_or_else(|| EmailError::invalid_smtp_url("SMTP URL must contain a password"))?
            .to_string();
        let host = url
            .host()
            .ok_or_else(|| EmailError::invalid_smtp_url("SMTP URL must contain a host"))?
            .to_string();
        // Port 25 is the default port for SMTP
        let port = url.port().unwrap_or(25);
        Ok(Self {
            host,
            port,
            username,
            password,
        })
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn creds(&self) -> Credentials {
        Credentials::new(self.username.clone(), self.password.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_config_from_env() -> Result<(), EmailError> {
        env::set_var("SMTP_URL", "smtps://username:password@localhost:1025");
        env::set_var("SMTP_FROM", "fake@email.com");
        env::set_var("MAILGUN_TEST_MODE", "true");
        let email_config = EmailConfig::from_env()?;
        assert_eq!(email_config.from(), "fake@email.com");
        assert!(email_config.smtp_connection().is_some());
        assert!(email_config.test_mode());

        // Now Test mode set to false
        env::set_var("MAILGUN_TEST_MODE", "false");
        let email_config = EmailConfig::from_env()?;
        assert!(!email_config.test_mode());

        // Now Test Mode set to non-boolean
        env::set_var("MAILGUN_TEST_MODE", "not_a_boolean");
        let email_config = EmailConfig::from_env()?;
        assert!(!email_config.test_mode());

        // Now missing
        env::remove_var("MAILGUN_TEST_MODE");
        let email_config = EmailConfig::from_env()?;
        assert!(!email_config.test_mode());

        // Now empty SMTP_URL
        env::set_var("SMTP_URL", "");
        let email_config = EmailConfig::from_env()?;
        assert!(email_config.smtp_connection.is_none());

        // Now missing
        env::remove_var("SMTP_URL");
        let email_config = EmailConfig::from_env()?;
        assert!(email_config.smtp_connection.is_none());

        // Now no SMTP_FROM
        env::remove_var("SMTP_FROM");
        let email_config = EmailConfig::from_env();
        assert!(email_config.is_err());

        Ok(())
    }

    #[test]
    fn smtp_connection() -> Result<(), EmailError> {
        let smtp_url = "smtps://user@user.com:pass@localhost:1025";
        let smtp_connection = SmtpConnection::new(smtp_url)?;
        assert_eq!(smtp_connection.host(), "localhost");
        assert_eq!(smtp_connection.port(), 1025);
        Ok(())
    }

    #[test]
    fn smtp_connection_no_port() -> Result<(), EmailError> {
        let smtp_url = "smtps://user:pass@localhost";
        let smtp_connection = SmtpConnection::new(smtp_url)?;
        assert_eq!(smtp_connection.host(), "localhost");
        assert_eq!(smtp_connection.port(), 25);
        Ok(())
    }

    #[test]
    fn smtp_connection_no_scheme() {
        let smtp_url = "user:pass@localhost";
        let smtp_connection = SmtpConnection::new(smtp_url);
        assert!(smtp_connection.is_err());
    }

    #[test]
    fn smtp_connection_invalid_scheme() {
        let smtp_url = "smtp://user:pass@localhost";
        let smtp_connection = SmtpConnection::new(smtp_url);
        assert!(smtp_connection.is_err());
    }

    #[test]
    fn smtp_connection_missing_host() {
        let smtp_url = ":1025?username:password";
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
