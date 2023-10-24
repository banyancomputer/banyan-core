use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub(crate) struct UserVariables {
    /// We attach a message id to the email when we send it. See `EmailMessage::build`
    #[serde(rename = "message-id")]
    pub(crate) message_id: Uuid,
}
