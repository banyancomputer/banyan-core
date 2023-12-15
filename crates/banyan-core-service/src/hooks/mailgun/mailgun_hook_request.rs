use serde::Deserialize;
use uuid::Uuid;

use crate::database::models::EmailMessageState;
use crate::hooks::mailgun::{EventData, MailgunHookError, Signature};

#[derive(Debug, Deserialize)]
pub struct MailgunHookRequest {
    #[serde(rename = "event-data")]
    event_data: EventData,
    signature: Signature,
}

impl MailgunHookRequest {
    pub fn event(&self) -> EmailMessageState {
        self.event_data.event().into()
    }

    pub fn message_id(&self) -> Uuid {
        self.event_data.message_id()
    }

    pub fn verify_signature(&self, key: &ring::hmac::Key) -> Result<(), MailgunHookError> {
        self.signature.verify(key)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hooks::mailgun::MailgunEvent;

    static JSON_DATA: &str = r#"
    {
        "signature": {
            "timestamp": "1529006854",
            "token": "a8ce0edb2dd8301dee6c2405235584e45aa91d1e9f979f3de0",
            "signature": "d2271d12299f6592d9d44cd9d250f0704e4674c30d79d07c47a66f95ce71cf55"
        },
        "event-data": {
            "event": "opened",
            "user-variables": {
                "message-id": "00000000-0000-0000-0000-000000000000"
            }
        }
    }
    "#;

    #[test]
    fn deserialize_mailgun_hook_payload() {
        let request: MailgunHookRequest = serde_json::from_str(JSON_DATA).unwrap();
        assert_eq!(request.event_data.event(), MailgunEvent::Opened);
        assert_eq!(request.event_data.message_id(), Uuid::nil());
    }
}
