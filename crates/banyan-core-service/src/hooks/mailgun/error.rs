use std::fmt::Display;

#[derive(Debug)]
#[non_exhaustive]
pub struct MailgunHookError {
    kind: MailgunHookErrorKind,
}

impl Display for MailgunHookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{:?}", self.kind))
    }
}

impl std::error::Error for MailgunHookError {}

impl MailgunHookError {
    pub fn failed_to_decode_signature(err: hex::FromHexError) -> Self {
        Self {
            kind: MailgunHookErrorKind::FailedToDecodeSignature(err),
        }
    }

    pub fn invalid_signature(err: ring::error::Unspecified) -> Self {
        Self {
            kind: MailgunHookErrorKind::InvalidSignature(err),
        }
    }
}

#[derive(Debug)]
pub enum MailgunHookErrorKind {
    /// Failed to decode signature (406)
    FailedToDecodeSignature(hex::FromHexError),
    /// Invalid Signature (406)
    InvalidSignature(ring::error::Unspecified),
}
