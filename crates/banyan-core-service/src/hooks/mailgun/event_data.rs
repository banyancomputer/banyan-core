use serde::Deserialize;
use uuid::Uuid;

use crate::hooks::mailgun::{MailgunEvent, UserVariables};

#[derive(Debug, Deserialize)]
pub(crate) struct EventData {
    event: MailgunEvent,

    #[serde(rename = "user-variables")]
    user_variables: UserVariables,
}

impl EventData {
    pub fn event(&self) -> MailgunEvent {
        self.event.clone()
    }

    pub fn message_id(&self) -> Uuid {
        self.user_variables.message_id
    }
}
