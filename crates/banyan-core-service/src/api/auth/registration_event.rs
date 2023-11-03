use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct RegistrationEvent {
    pub fingerprint: String,
    pub status: RegistrationEventStatus,
}

impl RegistrationEvent {
    pub fn approved(fingerprint: String, user_id: String) -> Self {
        Self {
            fingerprint,
            status: RegistrationEventStatus::Approved(user_id),
        }
    }

    pub fn rejected(fingerprint: String) -> Self {
        Self {
            fingerprint,
            status: RegistrationEventStatus::Rejected,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RegistrationEventStatus {
    Approved(String),
    Rejected,
}
