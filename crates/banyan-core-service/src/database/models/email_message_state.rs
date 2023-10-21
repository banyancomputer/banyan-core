use std::cmp::Ordering;
use std::fmt::{self, Display, Formatter};

use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
pub enum EmailMessageState {
    Queued,
    Sent,
    Accepted,
    Rejected,
    Delivered,
    Opened,
    Complained,
    Unsubscribed,
    Failed,
}

fn email_message_state_value(state: &EmailMessageState) -> i32 {
    match state {
        // Email is queued, but not sent
        EmailMessageState::Queued => 0,
        // Email is sent, but not accepted
        EmailMessageState::Sent => 1,
        // Email is either accepted or rejected
        EmailMessageState::Accepted => 2,
        EmailMessageState::Rejected => 2,
        // Email is delivered or failed
        EmailMessageState::Delivered => 3,
        EmailMessageState::Failed => 3,
        // Email is opened
        EmailMessageState::Opened => 4,
        EmailMessageState::Complained => 5,
        EmailMessageState::Unsubscribed => 6,
    }
}

impl PartialOrd for EmailMessageState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let self_value = email_message_state_value(self);
        let other_value = email_message_state_value(other);

        self_value.partial_cmp(&other_value)
    }
}

impl Display for EmailMessageState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            EmailMessageState::Queued => f.write_str("queued"),
            EmailMessageState::Sent => f.write_str("sent"),
            EmailMessageState::Accepted => f.write_str("accepted"),
            EmailMessageState::Delivered => f.write_str("delivered"),
            EmailMessageState::Opened => f.write_str("opened"),
            EmailMessageState::Complained => f.write_str("complained"),
            EmailMessageState::Unsubscribed => f.write_str("unsubscribed"),
            EmailMessageState::Failed => f.write_str("failed"),
            EmailMessageState::Rejected => f.write_str("rejected"),
        }
    }
}

impl From<String> for EmailMessageState {
    fn from(s: String) -> Self {
        match s.as_str() {
            "queued" => EmailMessageState::Queued,
            "sent" => EmailMessageState::Sent,
            "accepted" => EmailMessageState::Accepted,
            "delivered" => EmailMessageState::Delivered,
            "opened" => EmailMessageState::Opened,
            "complained" => EmailMessageState::Complained,
            "unsubscribed" => EmailMessageState::Unsubscribed,
            "failed" => EmailMessageState::Failed,
            "rejected" => EmailMessageState::Rejected,
            _ => panic!("invalid email message state: {}", s),
        }
    }
}
