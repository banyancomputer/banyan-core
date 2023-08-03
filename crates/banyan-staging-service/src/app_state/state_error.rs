use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
#[non_exhaustive]
pub struct StateError {
    kind: StateErrorKind,
}

impl StateError {
    pub(super) fn inaccessible_upload_directory(err: object_store::Error) -> Self {
        Self {
            kind: StateErrorKind::InaccessibleUploadDirectory(err),
        }
    }
}

impl Display for StateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use StateErrorKind::*;

        let msg = match self.kind {
            InaccessibleUploadDirectory(_) => "service upload directory isn't available",
        };

        f.write_str(msg)
    }
}

impl std::error::Error for StateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use StateErrorKind::*;

        match &self.kind {
            InaccessibleUploadDirectory(err) => Some(err),
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
enum StateErrorKind {
    InaccessibleUploadDirectory(object_store::Error),
}
