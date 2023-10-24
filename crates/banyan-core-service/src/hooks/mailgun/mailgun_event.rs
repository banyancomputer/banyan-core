use std::str::FromStr;

use serde::Deserializer;

use crate::database::models::EmailMessageState;
use crate::hooks::mailgun::MailgunHookError;

// A subset of EmailMessageState. These are the Mailgun events that we care about
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum MailgunEvent {
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
