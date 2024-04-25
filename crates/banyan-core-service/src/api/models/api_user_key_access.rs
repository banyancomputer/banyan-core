use crate::database::models::UserKeyAccess;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ApiUserKeyAccess {
    pub id: String,
    pub user_id: String,
    pub pem: String,
    pub fingerprint: String,
    pub bucket_ids: Vec<String>,
}

impl From<UserKeyAccess> for ApiUserKeyAccess {
    fn from(value: UserKeyAccess) -> Self {
        Self {
            id: value.id,
            user_id: value.user_id,
            pem: value.pem,
            fingerprint: value.fingerprint,
            bucket_ids: value
                .bucket_ids
                .split(",")
                .map(String::from)
                .collect::<Vec<_>>(),
        }
    }
}
