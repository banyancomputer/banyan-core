use std::ops::Deref;
use std::sync::Arc;

use axum::async_trait;
use axum::extract::{FromRef, FromRequestParts};
use http::request::Parts;

use crate::database::{Db, DbPool};

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
    db: Db,
}

#[async_trait]
impl DataSource for DbSource {
    async fn is_ready(&self) -> Result<(), DataSourceError> {
        match &self.db {
            #[cfg(feature = "postgres")]
            Db::Postgres(pdb) => {
                let mut _conn = pdb.direct().await.map_err(|_| DataSourceError::DependencyFailure)?;
                // Can't do this yet... Need to implement a passthrough Pool trait on the
                // TxExecutor for this to run...
                //let _ = sqlx::query_as!(i32, "SELECT 1 as id;").fetch_one(&mut conn).await.map_err(|_| DataSourceError::DependencyFailure)?;
            }
            #[cfg(feature = "sqlite")]
            Db::Sqlite(pdb) => {
                let _conn = pdb.direct().await.map_err(|_| DataSourceError::DependencyFailure)?;
            }
        }

        Ok(())
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for StateDataSource
where
    Db: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ();

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let db = Db::from_ref(state);
        Ok(StateDataSource(std::sync::Arc::new(DbSource { db })))
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
