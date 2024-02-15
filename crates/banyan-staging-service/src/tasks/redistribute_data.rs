use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike, TaskLikeExt, TaskStoreError};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::app::AppState;
use crate::clients::{
    CoreServiceClient, CoreServiceError, StorageProviderClient, StorageProviderError,
};
use crate::database::models::{Clients, Uploads};
use crate::tasks::upload_blocks::UploadBlocksTask;

pub type RedistributeDataTaskContext = AppState;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum RedistributeDataTaskError {
    #[error("sql error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("task store error: {0}")]
    TaskStoreError(#[from] TaskStoreError),

    #[error("core service error: {0}")]
    CoreServiceError(#[from] CoreServiceError),

    #[error("core service error: {0}")]
    StorageProviderError(#[from] StorageProviderError),

    #[error("http error: {0} response from {1}")]
    Http(http::StatusCode, Url),
}

#[derive(Deserialize, Serialize)]
pub struct RedistributeDataTask {
    metadata_id: String,
    new_host_id: String,
    new_host_url: String,
}

impl RedistributeDataTask {
    pub fn new(metadata_id: String, new_host_id: String, new_host_url: String) -> Self {
        Self {
            metadata_id,
            new_host_id,
            new_host_url,
        }
    }
}

#[async_trait]
impl TaskLike for RedistributeDataTask {
    const TASK_NAME: &'static str = "redistribute_data_task";

    type Error = RedistributeDataTaskError;
    type Context = RedistributeDataTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let mut database = ctx.database();
        let client = CoreServiceClient::new(
            ctx.secrets().service_signing_key(),
            ctx.service_name(),
            ctx.platform_name(),
            ctx.platform_hostname(),
        );
        let provider_credentials = client.request_provider_token(&self.new_host_id).await?;
        let storage_client =
            StorageProviderClient::new(&self.new_host_url, &provider_credentials.token);
        let upload = Uploads::get_by_metadata_id(&database, &self.metadata_id).await?;
        let client = Clients::find_by_upload_id(&database, &upload.id).await?;

        storage_client.push_client(client).await?;
        let new_upload = storage_client.new_upload(&self.metadata_id).await?;

        UploadBlocksTask {
            current_upload_id: upload.id.clone(),
            metadata_id: self.metadata_id.clone(),
            new_upload_id: new_upload.upload_id.clone(),
            storage_host_id: self.new_host_id.clone(),
            storage_host_url: self.new_host_url.clone(),
        }
        .enqueue::<banyan_task::SqliteTaskStore>(&mut database)
        .await?;
        Ok(())
    }

    fn unique_key(&self) -> Option<String> {
        Some(self.metadata_id.clone())
    }
}
