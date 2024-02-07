use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike, TaskLikeExt, TaskStoreError};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::app::AppState;
use crate::clients::storage_provider::{StorageProviderClient, StorageProviderError};
use crate::database::models::Clients;
use crate::tasks::upload_blocks::UploadBlocksTask;

pub type SetupUploadBlocksTaskContext = AppState;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum SetupUploadBlocksTaskError {
    #[error("sql error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("scheduling task error: {0}")]
    SchedulingTaskError(#[from] TaskStoreError),
    #[error("storage provider error: {0}")]
    StorageProviderError(#[from] StorageProviderError),
    #[error("http error: {0} response from {1}")]
    HttpError(http::StatusCode, Url),
}

#[derive(Deserialize, Serialize)]
pub struct SetupUploadBlocksTask {
    pub upload_id: String,
    pub metadata_id: String,
    pub storage_host: String,
    pub storage_authorization: String,
}

#[async_trait]
impl TaskLike for SetupUploadBlocksTask {
    const TASK_NAME: &'static str = "setup_upload_blocks_task";

    type Error = SetupUploadBlocksTaskError;
    type Context = SetupUploadBlocksTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let mut database = ctx.database();
        let storage_client =
            StorageProviderClient::new(&self.storage_host, &self.storage_authorization);

        let client = Clients::find_by_upload_id(&database, &self.upload_id).await?;
        // in order for the upload to get accepted
        let _ = storage_client.client_grant(&client.public_key).await?;
        let new_upload_response = storage_client.new_upload(&self.metadata_id).await?;

        UploadBlocksTask {
            current_upload_id: self.upload_id.clone(),
            new_upload_id: new_upload_response.upload_id.clone(),
            storage_host: self.storage_host.clone(),
            // TODO: would those expire?
            storage_authorization: self.storage_authorization.clone(),
        }
        .enqueue::<banyan_task::SqliteTaskStore>(&mut database)
        .await?;
        Ok(())
    }

    fn unique_key(&self) -> Option<String> {
        Some(self.upload_id.clone())
    }
}
