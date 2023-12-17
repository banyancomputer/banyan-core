use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike};
use jwt_simple::prelude::*;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::app::AppState;
use crate::database::models::Snapshot;

pub type CreateDealsTaskContext = AppState;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum CreateDealsTaskError {
    #[error("sql error: {0}")]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Deserialize, Serialize, Clone)]
pub struct PruneBlock {
    pub snapshot_id: String,
}

#[derive(Deserialize, Serialize)]
pub struct CreateDealsTask {
    snapshot_id: String,
}

impl CreateDealsTask {
    pub fn new(snapshot_id: String) -> Self {
        Self { snapshot_id }
    }
}

#[async_trait]
impl TaskLike for CreateDealsTask {
    const TASK_NAME: &'static str = "create_deals_task";

    type Error = CreateDealsTaskError;
    type Context = CreateDealsTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let mut transaction = ctx
            .database()
            .begin()
            .await
            .map_err(CreateDealsTaskError::Sqlx)?;

        let auth_key = ctx.secrets().service_key();

        // Determine where to send the prune list
        let storage_host_id = self.snapshot_id.to_string();
        let storage_host_info = sqlx::query_as!(
            Snapshot,
            "SELECT DISTINCT(id)
                FROM blocks AS b
                         JOIN block_locations AS bl ON bl.block_id = b.id
                         JOIN metadata AS m ON m.id = bl.metadata_id
                WHERE m.bucket_id = $1 AND bl.expired_at IS NULL
        "
        )
        .fetch_one(&mut *transaction)
        .await
        .map_err(CreateDealsTaskError::Sqlx)?;
        let storage_host_url = Url::parse(&storage_host_info.url)
            .map_err(|_| CreateDealsTaskError::Sqlx(sqlx::Error::RowNotFound))?;
        let storage_host_url = storage_host_url
            .join("/api/v1/core/prune")
            .map_err(|_| CreateDealsTaskError::Sqlx(sqlx::Error::RowNotFound))?;

        Ok(())
    }
}
