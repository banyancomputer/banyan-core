use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
#[non_exhaustive]
pub struct Error;

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("an unknown auth route error occurred")
    }
}

impl std::error::Error for Error {}