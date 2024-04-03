use serde::{Deserialize, Serialize};

use crate::database::models::UserKey;

#[derive(Serialize, Deserialize)]
pub struct ApiUserKey {
    pub id: String,
    pub name: String,
    pub user_id: String,

    pub api_access: bool,

    pub pem: String,
    pub fingerprint: String,
}

impl From<UserKey> for ApiUserKey {
    fn from(value: UserKey) -> Self {
        Self {
            id: value.id,
            name: value.name,
            user_id: value.user_id,
            api_access: value.api_access,
            pem: value.pem,
            fingerprint: value.fingerprint,
        }
    }
}
