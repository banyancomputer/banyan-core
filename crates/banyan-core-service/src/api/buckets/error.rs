use std::fmt::{Display, Formatter, Result};

#[derive(Debug)]
#[non_exhaustive]
pub struct Error;

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_str("an unknown bucket route error occurred")
    }
}

impl std::error::Error for Error {}
