use std::sync::OnceLock;

use regex::Regex;

pub mod authenticated_client;
mod block_reader;
pub mod platform_identity;
pub mod storage_grant;
pub mod upload_store;

/// Defines the maximum length of time we consider any individual token valid in seconds. If the
/// expiration is still in the future, but it was issued more than this many seconds in the past
/// we'll reject the token even if its otherwise valid.
const MAXIMUM_TOKEN_AGE: u64 = 900;

static PAIRED_ID_PATTERN: &str =
    r"^([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})@([0-9a-f]{40})$";

static PAIRED_ID_VALIDATOR: OnceLock<Regex> = OnceLock::new();

static FINGERPRINT_PATTERN: &str = r"^[0-9a-f]{40}$";

static FINGERPRINT_VALIDATOR: OnceLock<Regex> = OnceLock::new();

pub fn fingerprint_validator() -> &'static Regex {
    FINGERPRINT_VALIDATOR.get_or_init(|| Regex::new(FINGERPRINT_PATTERN).unwrap())
}

pub fn paired_id_validator() -> &'static Regex {
    PAIRED_ID_VALIDATOR.get_or_init(|| Regex::new(PAIRED_ID_PATTERN).unwrap())
}

pub use authenticated_client::AuthenticatedClient;
pub use block_reader::BlockReader;
pub use platform_identity::PlatformIdentity;
pub use storage_grant::StorageGrant;