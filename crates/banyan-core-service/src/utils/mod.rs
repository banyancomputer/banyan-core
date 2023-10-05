//pub mod car_buffer;
//pub mod db;
//pub mod keys;
//pub mod metadata_upload;
//pub mod storage_ticket;

use std::error::Error;

pub fn collect_error_messages(base_error: impl Error) -> Vec<String> {
    let mut errors = vec![base_error.to_string()];
    let mut source = base_error.source();

    while let Some(err) = source {
        errors.push(err.to_string());
        source = err.source();
    }

    errors
}
