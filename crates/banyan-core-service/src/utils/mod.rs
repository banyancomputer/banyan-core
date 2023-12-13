pub mod car_buffer;
pub mod keys;

use std::error::Error;
use std::str::FromStr;

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
    cid::Cid::from_str(cid)?.to_string_of_base(cid::multibase::Base::Base64Url)
}
