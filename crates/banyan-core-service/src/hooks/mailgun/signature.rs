use serde::Deserialize;

use crate::hooks::mailgun::MailgunHookError;

#[derive(Debug, Deserialize)]
pub(crate) struct Signature {
    signature: String,
    timestamp: String,
    token: String,
}

impl Signature {
    pub(crate) fn verify(&self, key: &ring::hmac::Key) -> Result<(), MailgunHookError> {
        let data = format!("{}{}", self.timestamp, self.token);

        let signature =
            hex::decode(&self.signature).map_err(MailgunHookError::FailedToDecodeSignature)?;

        ring::hmac::verify(key, data.as_bytes(), &signature)
            .map_err(MailgunHookError::InvalidSignature)
    }
}
