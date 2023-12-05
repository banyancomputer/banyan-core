use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    TaskStoreError(banyan_task::TaskStoreError),
}

impl Error {
    pub fn is_temporary(&self) -> bool {
        false
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_str("an unknown healthcheck route error occurred")
    }
}

impl std::error::Error for Error {}
