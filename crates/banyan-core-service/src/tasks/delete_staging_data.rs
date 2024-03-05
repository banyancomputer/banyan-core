use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::app::AppState;
use crate::clients::{DeleteBlocksRequest, StagingServiceClient, StagingServiceError};
use crate::database::models::StorageHost;

#[derive(Deserialize, Serialize)]
pub struct DeleteStagingDataTask {
    normalized_cids: Vec<String>,
    metadata_id: String,
    grant_id: String,
}

impl DeleteStagingDataTask {
    pub fn new(metadata_id: String, normalized_cids: Vec<String>, grant_id: String) -> Self {
        Self {
            metadata_id,
            normalized_cids,
            grant_id,
        }
    }
}
#[async_trait]
impl TaskLike for DeleteStagingDataTask {
    const TASK_NAME: &'static str = "delete_staging_data_task";

    type Error = DeleteStagingDataTaskError;
    type Context = AppState;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let database = ctx.database();
        let staging_host = StorageHost::select_staging(&database).await?;

        let staging_client = StagingServiceClient::new(
            ctx.secrets().service_key(),
            ctx.service_name(),
            staging_host.name.as_str(),
            Url::parse(&staging_host.url)?,
        );

        staging_client
            .delete_blocks(DeleteBlocksRequest {
                metadata_id: self.metadata_id.clone(),
                normalized_cids: self.normalized_cids.clone(),
                grant_id: self.grant_id.clone(),
            })
            .await?;

        Ok(())
    }

    fn unique_key(&self) -> Option<String> {
        Some(self.metadata_id.clone())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteStagingDataTaskError {
    #[error("sql error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("staging host url error: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error("jwt error: {0}")]
    JwtError(#[from] jwt_simple::Error),
    #[error("staging error: {0}")]
    StagingServiceError(#[from] StagingServiceError),
}
