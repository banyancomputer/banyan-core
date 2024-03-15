pub mod car_buffer;
pub mod keys;
#[cfg(test)]
pub mod tests;
pub mod time;

use std::error::Error;
use std::str::FromStr;

use crate::database::models::UserStorageReport;

pub const ONE_HUNDRED_MIB: i64 = 100 * 1024 * 1024;

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

pub fn rounded_storage_authorization(report: &UserStorageReport, additional_capacity: i64) -> i64 {
    let new_required_amount = report.current_consumption() + additional_capacity;
    // Integer division always rounds down, we want to round up to the nearest 100MiB
    ((new_required_amount / ONE_HUNDRED_MIB) + 1) * ONE_HUNDRED_MIB
}

pub fn minimal_grant_amount() -> i64 {
    ONE_HUNDRED_MIB
}
