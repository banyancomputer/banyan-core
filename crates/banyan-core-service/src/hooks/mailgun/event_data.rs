use serde::Deserialize;
use uuid::Uuid;

use crate::hooks::mailgun::{MailgunEvent, UserVariables};

#[derive(Debug, Deserialize)]
pub(crate) struct EventData {
    event: MailgunEvent,

    #[serde(rename = "user-variables")]
    user_variables: Option<UserVariables>,
}

impl EventData {
    pub fn event(&self) -> MailgunEvent {
        self.event.clone()
    }

    pub fn message_id(&self) -> Option<Uuid> {
        self.user_variables.as_ref().map(|uv| uv.message_id.clone())
    }
}
