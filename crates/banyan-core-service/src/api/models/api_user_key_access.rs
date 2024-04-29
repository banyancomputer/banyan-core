use serde::{Deserialize, Serialize};

use super::ApiUserKey;
use crate::database::models::UserKeyAccess;

#[derive(Serialize, Deserialize)]
pub struct ApiUserKeyAccess {
    pub key: ApiUserKey,
    pub bucket_ids: Vec<String>,
}

impl From<UserKeyAccess> for ApiUserKeyAccess {
    fn from(value: UserKeyAccess) -> Self {
        let bucket_ids = if value.bucket_ids.is_empty() {
            vec![]
        } else {
            value
                .bucket_ids
                .split(',')
                .map(String::from)
                .collect::<Vec<_>>()
        };
        Self {
            key: ApiUserKey {
                id: value.id,
                name: value.name,
                user_id: value.user_id,
                api_access: value.api_access,
                pem: value.pem,
                fingerprint: value.fingerprint,
                created_at: value.created_at,
            },
            bucket_ids,
        }
    }
}
