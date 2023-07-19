use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
#[non_exhaustive]
pub struct StateError {
    kind: StateErrorKind,
}

impl StateError {
    pub(super) fn bad_database_url(err: sqlx::Error) -> Self {
        Self {
            kind: StateErrorKind::BadDatabaseUrl(err),
        }
    }

    pub(super) fn database_unavailable(err: sqlx::Error) -> Self {
        Self {
            kind: StateErrorKind::DatabaseUnavailable(err),
        }
    }

    pub(super) fn inaccessible_upload_directory(err: object_store::Error) -> Self {
        Self {
            kind: StateErrorKind::InaccessibleUploadDirectory(err),
        }
    }

    pub(super) fn invalid_service_key() -> Self {
        Self {
            kind: StateErrorKind::InvalidServiceKey,
        }
    }

    pub(super) fn migrations_failed(err: sqlx::migrate::MigrateError) -> Self {
        Self {
            kind: StateErrorKind::MigrationsFailed(err),
        }
    }

    pub(super) fn read_service_key(err: std::io::Error) -> Self {
        Self {
            kind: StateErrorKind::ReadServiceKey(err),
        }
    }

    pub(super) fn service_keygen_failed() -> Self {
        Self {
            kind: StateErrorKind::ServiceKeygenFailed,
        }
    }

    pub(super) fn write_service_key(err: std::io::Error) -> Self {
        Self {
            kind: StateErrorKind::WriteServiceKey(err),
        }
    }
}

impl Display for StateError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use StateErrorKind::*;

        let msg = match self.kind {
            BadDatabaseUrl(_) => "provided database URL wasn't usable",
            DatabaseUnavailable(_) => "unable to make use of the configured database",
            InaccessibleUploadDirectory(_) => "service upload directory isn't available",
            InvalidServiceKey => "provided service key is not a valid format",
            MigrationsFailed(_) => "failed to run migrations against configured database",
            ReadServiceKey(_) => "unable to read service key from provided location",
            ServiceKeygenFailed => "unable to create new ECDSA service key",
            WriteServiceKey(_) => "unable to write service key to provided location",
        };

        f.write_str(msg)
    }
}

impl std::error::Error for StateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        use StateErrorKind::*;

        match &self.kind {
            BadDatabaseUrl(err) => Some(err),
            DatabaseUnavailable(err) => Some(err),
            InaccessibleUploadDirectory(err) => Some(err),
            MigrationsFailed(err) => Some(err),
            // todo: more types through here
            _ => None,
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
enum StateErrorKind {
    BadDatabaseUrl(sqlx::Error),
    DatabaseUnavailable(sqlx::Error),
    InaccessibleUploadDirectory(object_store::Error),
    InvalidServiceKey,
    MigrationsFailed(sqlx::migrate::MigrateError),
    ReadServiceKey(std::io::Error),
    ServiceKeygenFailed,
    WriteServiceKey(std::io::Error),
}
