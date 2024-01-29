use jwt_simple::prelude::{Deserialize, Serialize};

use crate::database::models::SelectedStorageHost;

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
impl From<SelectedStorageHost> for ApiSelectedStorageHostAdmin {
    fn from(value: SelectedStorageHost) -> Self {
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
