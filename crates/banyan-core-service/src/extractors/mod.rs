mod api_identity;
mod data_store;
mod server_base;
mod session_identity;
mod signing_key;
mod storage_host_token;

pub use api_identity::{ApiIdentity, EXPIRATION_WINDOW_SECS};
pub use data_store::DataStore;
pub use server_base::ServerBase;
pub use session_identity::SessionIdentity;
pub use signing_key::SigningKey;
pub use storage_host_token::{
    StorageHostToken, EXPIRATION_WINDOW_SECS as STORAGE_HOST_EXPIRATION_WINDOW_SECS,
};

pub static LOGIN_PATH: &str = "/auth/login";

pub static SESSION_COOKIE_NAME: &str = "_session_id";
