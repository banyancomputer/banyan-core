use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::database::models::UserKey;

#[derive(Serialize, Deserialize)]
pub struct ApiUserKey {
    pub id: String,
    pub name: String,
    pub user_id: String,

    pub api_access: bool,

    pub public_key: String,
    pub fingerprint: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

impl From<UserKey> for ApiUserKey {
    fn from(value: UserKey) -> Self {
        Self {
            id: value.id,
            name: value.name,
            user_id: value.user_id,
            api_access: value.api_access,
            public_key: value.public_key,
            fingerprint: value.fingerprint,
            created_at: value.created_at,
        }
    }
}