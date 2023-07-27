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

    pub(super) fn key_loading(err: openssl::error::ErrorStack) -> Self {
        Self {
            kind: StateErrorKind::KeyLoading(err),
        }
    }

    pub(super) fn loading_state_keys(err: jsonwebtoken::errors::Error) -> Self {
        Self {
            kind: StateErrorKind::LoadingStateKeys(err),
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

    pub(super) fn service_keygen_failed(err: openssl::error::ErrorStack) -> Self {
        Self {
            kind: StateErrorKind::ServiceKeygenFailed(err),
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
            KeyLoading(_) => "unable to load service keys",
            LoadingStateKeys(_) => "unable to load service key into signing and verification keys",
            MigrationsFailed(_) => "failed to run migrations against configured database",
            ReadServiceKey(_) => "unable to read service key from provided location",
            ServiceKeygenFailed(_) => "unable to create new ECDSA service key",
            WriteServiceKey(_) => "unable to persist geneated service key to disk",
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
            KeyLoading(err) => Some(err),
            LoadingStateKeys(err) => Some(err),
            ReadServiceKey(err) => Some(err),
            ServiceKeygenFailed(err) => Some(err),
            WriteServiceKey(err) => Some(err),
        }
    }
}

#[derive(Debug)]
#[non_exhaustive]
enum StateErrorKind {
    BadDatabaseUrl(sqlx::Error),
    DatabaseUnavailable(sqlx::Error),
    InaccessibleUploadDirectory(object_store::Error),
    KeyLoading(openssl::error::ErrorStack),
    LoadingStateKeys(jsonwebtoken::errors::Error),
    MigrationsFailed(sqlx::migrate::MigrateError),
    ReadServiceKey(std::io::Error),
    ServiceKeygenFailed(openssl::error::ErrorStack),
    WriteServiceKey(std::io::Error),
}
