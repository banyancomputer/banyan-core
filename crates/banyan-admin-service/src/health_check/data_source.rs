use std::ops::Deref;
use std::sync::Arc;

use axum::async_trait;
use axum::extract::{FromRef, FromRequestParts};
use http::request::Parts;
use serde::Serialize;

use crate::database::Database;

// TODO: this should be generic for metrics we would want to collect over a data source.
#[derive(Debug, Serialize)]
pub struct DataSourceMetrics {}

impl DataSourceMetrics {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
pub trait DataSource {
    /// Perform various checks on the system to ensure its healthy and ready to accept requests.
    async fn is_ready(&self) -> Result<DataSourceMetrics, DataSourceError>;
}

#[derive(Debug, thiserror::Error)]
pub enum DataSourceError {
    #[error("one or more dependent services aren't available")]
    DependencyFailure,
}

pub type DynDataSource = Arc<dyn DataSource + Send + Sync>;

pub struct StateDataSource(DynDataSource);

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
    async fn is_ready(&self) -> Result<DataSourceMetrics, DataSourceError> {
        // Simple check to ensure the database is up and running.
        let _ = sqlx::query("SELECT 1 as id;")
            .fetch_one(&self.db)
            .await
            .map_err(|_| DataSourceError::DependencyFailure)?;

        Ok(DataSourceMetrics::new())
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for StateDataSource
where
    Database: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ();

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Ok(StateDataSource(Arc::new(DbSource {
            db: Database::from_ref(state),
        })))
    }
}
