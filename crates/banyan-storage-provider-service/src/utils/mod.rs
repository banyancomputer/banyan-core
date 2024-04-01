use std::sync::OnceLock;

mod keys;

pub use keys::{fingerprint_key_pair, fingerprint_public_key, SigningKey, VerificationKey};

static CID_VALIDATOR: OnceLock<regex::Regex> = OnceLock::new();

const CID_REGEX: &str = r"^u([A-Za-z0-9_-]{48}|[A-Za-z0-9_-]{59})$";

pub fn is_valid_cid(cid: &str) -> bool {
    let re = CID_VALIDATOR.get_or_init(|| regex::Regex::new(CID_REGEX).unwrap());
    re.is_match(cid)
}
