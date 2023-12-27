pub mod car_buffer;
pub mod keys;
#[cfg(test)]
pub mod tests;

use std::error::Error;
use std::str::FromStr;

/// This is the CID representation that is standardized internally. We should be able to receive
/// CIDs in various compatible formats and always normalize to this variant before comparing or
/// interacting with and internal CID references.
pub const NORMALIZED_CID_BASE: cid::multibase::Base = cid::multibase::Base::Base64Url;

pub fn collect_error_messages(base_error: impl Error) -> Vec<String> {
    let mut errors = vec![base_error.to_string()];
    let mut source = base_error.source();

    while let Some(err) = source {
        errors.push(err.to_string());
        source = err.source();
    }

    errors
}

pub fn normalize_cid(cid: &str) -> Result<String, cid::Error> {
    cid::Cid::from_str(cid)?.to_string_of_base(NORMALIZED_CID_BASE)
}
