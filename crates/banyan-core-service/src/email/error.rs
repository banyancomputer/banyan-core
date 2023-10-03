use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
#[non_exhaustive]
pub struct EmailError {
    kind: EmailErrorKind,
}

impl EmailError {
    pub fn default_error(message: &str) -> Self {
        Self {
            kind: EmailErrorKind::Default(message.to_string()),
        }
    }

    pub fn message_build_error(err: lettre::error::Error) -> Self {
        Self {
            kind: EmailErrorKind::MessageBuildError(err),
        }
    }

    pub fn smtp_transport_build_error(err: lettre::transport::smtp::Error) -> Self {
        Self {
            kind: EmailErrorKind::SmtpTransportBuildError(err),
        }
    }

    pub fn render_error(err: handlebars::RenderError) -> Self {
        Self {
            kind: EmailErrorKind::RenderError(err),
        }
    }

    pub fn stub_send_error(err: lettre::transport::stub::Error) -> Self {
        Self {
            kind: EmailErrorKind::StubSendError(err),
        }
    }

    pub fn smtp_send_error(err: lettre::transport::smtp::Error) -> Self {
        Self {
            kind: EmailErrorKind::SmtpSendError(err),
        }
    }

    pub fn missing_smtp_from() -> Self {
        Self {
            kind: EmailErrorKind::MissingSmtpFrom,
        }
    }

    pub fn invalid_smtp_url(url: &str) -> Self {
        Self {
            kind: EmailErrorKind::InvalidSmptUrl(url.to_string()),
        }
    }

    pub fn invalid_smtp_from(err: lettre::address::AddressError) -> Self {
        Self {
            kind: EmailErrorKind::InvalidSmtpFrom(err),
        }
    }

    pub fn invalid_smtp_to(err: lettre::address::AddressError) -> Self {
        Self {
            kind: EmailErrorKind::InvalidSmtpTo(err),
        }
    }

    pub fn invalid_test_mode(err: std::str::ParseBoolError) -> Self {
        Self {
            kind: EmailErrorKind::InvalidTestMode(err),
        }
    }

    pub fn utf8_error(err: std::str::Utf8Error) -> Self {
        Self {
            kind: EmailErrorKind::Utf8Error(err),
        }
    }
}

impl Display for EmailError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("{:?}", self.kind))
    }
}

impl std::error::Error for EmailError {}

#[derive(Debug)]
#[non_exhaustive]
enum EmailErrorKind {
    Default(String),
    MessageBuildError(lettre::error::Error),
    SmtpTransportBuildError(lettre::transport::smtp::Error),
    RenderError(handlebars::RenderError),
    StubSendError(lettre::transport::stub::Error),
    SmtpSendError(lettre::transport::smtp::Error),
    MissingSmtpFrom,
    InvalidSmptUrl(String),
    InvalidSmtpFrom(lettre::address::AddressError),
    InvalidSmtpTo(lettre::address::AddressError),
    InvalidTestMode(std::str::ParseBoolError),
    Utf8Error(std::str::Utf8Error),
}
