use jwt_simple::prelude::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ApiStorageHostAdmin {
    pub id: String,
    pub name: String,
    pub url: String,
    pub used_storage: i64,
    pub available_storage: i64,
    pub fingerprint: String,
    pub pem: String,
}