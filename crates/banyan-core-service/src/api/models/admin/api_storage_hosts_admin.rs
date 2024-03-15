use jwt_simple::prelude::{Deserialize, Serialize};

use crate::database::models::StorageHost;

#[derive(Serialize, Deserialize)]
pub struct ApiSelectedStorageHostAdmin {
    pub id: String,
    pub name: String,
    pub url: String,
    pub used_storage: i64,
    pub available_storage: i64,
    pub fingerprint: String,
    pub pem: String,
}
impl From<StorageHost> for ApiSelectedStorageHostAdmin {
    fn from(value: StorageHost) -> Self {
        Self {
            id: value.id,
            name: value.name,
            url: value.url,
            used_storage: value.used_storage,
            available_storage: value.available_storage,
            fingerprint: value.fingerprint,
            pem: value.pem,
        }
    }
}
