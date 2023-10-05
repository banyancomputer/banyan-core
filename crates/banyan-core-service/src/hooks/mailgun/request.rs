use std::str::FromStr;

use ring::hmac::Key as HmacKey;
use serde::{de::Deserializer, Deserialize};
use uuid::Uuid;

use crate::db::models::EmailMessageState;

use super::error::MailgunHookError;

#[derive(Debug, Deserialize)]
pub struct MailgunHookRequest {
    signature: Signature,
    #[serde(rename = "event-data")]
    event_data: EventData,
}

impl MailgunHookRequest {
    pub fn verify(&self, key: &HmacKey) -> Result<(), MailgunHookError> {
        self.signature.verify(key)?;
        Ok(())
    }

    pub fn message_id(&self) -> Uuid {
        self.event_data.message_id()
    }

    pub fn event(&self) -> MailgunEvent {
        self.event_data.event()
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
        // TODO: this is probably not the best way to decode the signature
        match ring::hmac::verify(key, data.as_bytes(), self.signature.as_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(MailgunHookError::invalid_signature()),
        }
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

#[derive(Debug, Deserialize)]
struct UserVariables {
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
            _ => Err(MailgunHookError::invalid_event()),
        }
    }
}
