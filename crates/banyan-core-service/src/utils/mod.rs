pub mod car_buffer;
pub mod keys;
#[cfg(test)]
pub mod tests;
pub mod time;

use std::error::Error;
use std::sync::OnceLock;

use crate::database::models::UserStorageReport;

static CID_VALIDATOR: OnceLock<regex::Regex> = OnceLock::new();

const CID_REGEX: &str = r"^u([A-Za-z0-9_-]{48}|[A-Za-z0-9_-]{96})$";

pub(crate) const GIBIBYTE: i64 = 1024 * 1024 * 1024;

pub(crate) const ONE_HUNDRED_MIB: i64 = 100 * 1024 * 1024;

pub fn collect_error_messages(base_error: impl Error) -> Vec<String> {
    let mut errors = vec![base_error.to_string()];
    let mut source = base_error.source();

    while let Some(err) = source {
        errors.push(err.to_string());
        source = err.source();
    }

    errors
}

pub fn is_valid_cid(cid: &str) -> bool {
    let re = CID_VALIDATOR.get_or_init(|| regex::Regex::new(CID_REGEX).unwrap());
    re.is_match(cid)
}

pub fn minimal_grant_amount() -> i64 {
    ONE_HUNDRED_MIB
}

pub fn rounded_storage_authorization(report: &UserStorageReport, additional_capacity: i64) -> i64 {
    let new_required_amount = report.current_consumption() + additional_capacity;
    // Integer division always rounds down, we want to round up to the nearest 100MiB
    ((new_required_amount / ONE_HUNDRED_MIB) + 1) * ONE_HUNDRED_MIB
}
