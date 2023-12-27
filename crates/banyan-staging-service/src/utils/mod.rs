mod keys;

pub use keys::{fingerprint_key_pair, fingerprint_public_key, SigningKey, VerificationKey};

/// This is the CID representation that is standardized internally. We should be able to receive
/// CIDs in various compatible formats and always normalize to this variant before comparing or
/// interacting with and internal CID references.
pub const NORMALIZED_CID_BASE: cid::multibase::Base = cid::multibase::Base::Base64Url;
