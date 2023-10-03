use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
#[non_exhaustive]
pub struct EmailError {
    kind: EmailErrorKind,
}

impl EmailError {
    pub fn default_error(message: &str) -> Self {
        Self {
            kind: EmailErrorKind::Default(message.to_string()),
        }
    }
}

impl Display for EmailError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("{:?}", self.kind))
    }
}

impl std::error::Error for EmailError {}

#[derive(Debug)]
#[non_exhaustive]
enum EmailErrorKind {
    Default(String), 
}
