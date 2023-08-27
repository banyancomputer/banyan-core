use std::ops::Deref;
use std::sync::Arc;

use axum::async_trait;
use axum::extract::{FromRef, FromRequestParts};
use http::request::Parts;

use crate::database::{Database, Executor};

#[async_trait]
pub trait DataSource {
    /// Perform various checks on the system to ensure its healthy and ready to accept requests.
    async fn is_ready(&self) -> Result<(), DataSourceError>;
}

#[derive(Debug, thiserror::Error)]
pub enum DataSourceError {
    #[error("one or more dependent services aren't available")]
    DependencyFailure,

    #[error("service has received signal indicating it should shutdown")]
    ShuttingDown,
}

pub type DynDataSource = Arc<dyn DataSource + Send + Sync>;

pub struct StateDataSource(DynDataSource);

impl StateDataSource {
    #[cfg(test)]
    pub fn new(dds: DynDataSource) -> Self {
        Self(dds)
    }
}

impl Deref for StateDataSource {
    type Target = DynDataSource;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

struct DbSource {
    db: Database,
}

#[async_trait]
impl DataSource for DbSource {
    async fn is_ready(&self) -> Result<(), DataSourceError> {
        match self.db.ex() {
            #[cfg(feature = "postgres")]
            Executor::Postgres(ref mut conn) => {
                let _ = sqlx::query("SELECT 1 as id;")
                    .fetch_one(conn)
                    .await
                    .map_err(|_| DataSourceError::DependencyFailure)?;
            },

            #[cfg(feature = "sqlite")]
            Executor::Sqlite(ref mut conn) => {
                let _ = sqlx::query("SELECT 1 as id;")
                    .fetch_one(conn)
                    .await
                    .map_err(|_| DataSourceError::DependencyFailure)?;
            },
        }

        Ok(())
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for StateDataSource
where
    Database: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ();

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        Ok(StateDataSource(Arc::new(DbSource { db: Database::from_ref(state) })))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[derive(Clone)]
    pub(crate) enum MockReadiness {
        DependencyFailure,
        Ready,
        ShuttingDown,
    }

    #[async_trait]
    impl DataSource for MockReadiness {
        async fn is_ready(&self) -> Result<(), DataSourceError> {
            use MockReadiness::*;

            match self {
                DependencyFailure => Err(DataSourceError::DependencyFailure),
                Ready => Ok(()),
                ShuttingDown => Err(DataSourceError::ShuttingDown),
            }
        }
    }
}
