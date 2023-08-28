use std::sync::OnceLock;

use regex::Regex;

pub mod authenticated_client;
pub mod database;
pub mod storage_grant;
pub mod upload_store;

static KEY_ID_PATTERN: &str = r"^([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})@([0-9a-f]{2}(:[0-9a-f]{2}){19})$";

static KEY_ID_VALIDATOR: OnceLock<Regex> = OnceLock::new();

pub fn key_validator() -> &'static Regex {
    KEY_ID_VALIDATOR.get_or_init(|| Regex::new(KEY_ID_PATTERN).unwrap())
}

pub use authenticated_client::AuthenticatedClient;
pub use database::Database;
pub use storage_grant::StorageGrant;
pub use upload_store::UploadStore;
