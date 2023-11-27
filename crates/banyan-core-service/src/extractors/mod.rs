#![allow(dead_code)]

mod api_identity;
mod data_store;
mod server_base;
mod session_identity;
mod signing_key;
mod storage_provider_identity;
mod user_identity;

pub use data_store::DataStore;
pub use server_base::ServerBase;
pub use session_identity::SessionIdentity;
pub use storage_provider_identity::StorageProviderIdentity;
pub use user_identity::UserIdentity;

use std::sync::OnceLock;

// Allow 15 minute token windows for now, this is likely to change in the future
pub const EXPIRATION_WINDOW_SECS: u64 = 900;

static KEY_ID_VALIDATOR: OnceLock<regex::Regex> = OnceLock::new();

const KEY_ID_REGEX: &str = r"^[0-9a-f]{40}$";
