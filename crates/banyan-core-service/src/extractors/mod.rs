#![allow(dead_code)]

mod api_identity;
mod data_store;
mod server_base;
mod session_identity;
mod signing_key;
mod storage_provider_identity;
mod user_identity;

pub use api_identity::ApiIdentity;
pub use data_store::DataStore;
pub use server_base::ServerBase;
pub use session_identity::SessionIdentity;
pub use storage_provider_identity::StorageProviderIdentity;
pub use user_identity::{Identity, UserIdentity};

pub static LOGIN_PATH: &str = "/auth/login";
