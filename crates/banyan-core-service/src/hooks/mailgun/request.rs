use std::str::FromStr;

use ring::hmac::Key as HmacKey;
use serde::{de::Deserializer, Deserialize};
use uuid::Uuid;

use crate::db::models::EmailMessageState;

use super::error::MailgunHookError;

/// Form of a Mailgun webhook request
/// All such requests are POST requests
#[derive(Debug, Deserialize)]
pub struct MailgunHookRequest {
    signature: Signature,
    #[serde(rename = "event-data")]
    event_data: EventData,
}

impl MailgunHookRequest {
    /// Verify the signature of the request
    pub fn verify_signature(&self, key: &HmacKey) -> Result<(), MailgunHookError> {
        self.signature.verify(key)?;
        Ok(())
    }

    /// Extract our custom message id from the request
    pub fn message_id(&self) -> Uuid {
        self.event_data.message_id()
    }

    /// Extract the event from the request
    pub fn event(&self) -> EmailMessageState {
        self.event_data.event().into()
    }
}

#[derive(Debug, Deserialize)]
pub struct Signature {
    timestamp: String,
    token: String,
    signature: String,
}

impl Signature {
    pub fn verify(&self, key: &HmacKey) -> Result<(), MailgunHookError> {
        let data = format!("{}{}", self.timestamp, self.token);
        let signature =
            hex::decode(&self.signature).map_err(MailgunHookError::failed_to_decode_signature)?;
        ring::hmac::verify(key, data.as_bytes(), &signature)
            .map_err(MailgunHookError::invalid_signature)
    }
}

#[derive(Debug, Deserialize)]
struct EventData {
    event: MailgunEvent,
    #[serde(rename = "user-variables")]
    user_variables: UserVariables,
}

impl EventData {
    pub fn message_id(&self) -> Uuid {
        self.user_variables.message_id
    }

    pub fn event(&self) -> MailgunEvent {
        self.event.clone()
    }
}

/// Our custom user variables:
#[derive(Debug, Deserialize)]
struct UserVariables {
    /// We attach a message id to the email when we send it. See `EmailMessage::build`
    #[serde(rename = "message-id")]
    message_id: Uuid,
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(request.event_data.event, MailgunEvent::Opened);
        assert_eq!(request.event_data.user_variables.message_id, Uuid::nil());
    }
}

// A subset of EmailMessageState. These are the Mailgun events that we care about
#[derive(Debug, Clone, PartialEq)]
pub enum MailgunEvent {
    Accepted,
    Rejected,
    Delivered,
    Failed,
    Opened,
    Unsubscribed,
    Complained,
}

impl From<MailgunEvent> for EmailMessageState {
    fn from(event: MailgunEvent) -> Self {
        match event {
            MailgunEvent::Accepted => EmailMessageState::Accepted,
            MailgunEvent::Rejected => EmailMessageState::Rejected,
            MailgunEvent::Delivered => EmailMessageState::Delivered,
            MailgunEvent::Failed => EmailMessageState::Failed,
            MailgunEvent::Opened => EmailMessageState::Opened,
            MailgunEvent::Unsubscribed => EmailMessageState::Unsubscribed,
            MailgunEvent::Complained => EmailMessageState::Complained,
        }
    }
}

impl<'de> serde::Deserialize<'de> for MailgunEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        MailgunEvent::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl FromStr for MailgunEvent {
    type Err = MailgunHookError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "accepted" => Ok(MailgunEvent::Accepted),
            "rejected" => Ok(MailgunEvent::Rejected),
            "delivered" => Ok(MailgunEvent::Delivered),
            "failed" => Ok(MailgunEvent::Failed),
            "opened" => Ok(MailgunEvent::Opened),
            "unsubscribed" => Ok(MailgunEvent::Unsubscribed),
            "complained" => Ok(MailgunEvent::Complained),
            _ => panic!("invalid event"),
        }
    }
}
