use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike, TaskLikeExt, TaskStoreError};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::app::AppState;
use crate::clients::{
    ClientsRequest, CoreServiceClient, CoreServiceError, NewUploadRequest, StorageProviderClient,
    StorageProviderError,
};
use crate::tasks::replicate_blocks::ReplicateBlocksTask;

pub type ReplicateDataTaskContext = AppState;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum ReplicateDataTaskError {
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
pub struct ReplicateDataTask {
    pub metadata_id: String,
    pub block_cids: Vec<String>,
    pub new_host_id: String,
    pub new_host_url: String,
    pub new_storage_grant_id: String,
    pub new_storage_grant_size: i64,
    pub old_host_id: String,
    pub old_host_url: String,
}

#[async_trait]
impl TaskLike for ReplicateDataTask {
    const TASK_NAME: &'static str = "replicate_data_task";

    type Error = ReplicateDataTaskError;
    type Context = ReplicateDataTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let database = ctx.database();
        let client = CoreServiceClient::new(
            ctx.secrets().service_signing_key(),
            ctx.service_name(),
            ctx.platform_name(),
            ctx.platform_hostname(),
        )?;
        let old_provider_credentials = client.request_provider_token(&self.old_host_id).await?;
        let old_storage_client =
            StorageProviderClient::new(&self.old_host_url, &old_provider_credentials.token);
        let new_provider_credentials = client.request_provider_token(&self.new_host_id).await?;
        let new_storage_client =
            StorageProviderClient::new(&self.new_host_url, &new_provider_credentials.token);

        let existing_client = old_storage_client.get_client(&self.metadata_id).await?;
        let new_client = new_storage_client
            .push_client(ClientsRequest {
                platform_id: existing_client.platform_id,
                fingerprint: existing_client.fingerprint,
                public_key: existing_client.public_key,
            })
            .await?;

        let new_upload = new_storage_client
            .new_upload(&NewUploadRequest {
                metadata_id: self.metadata_id.clone(),
                client_id: new_client.id.clone(),
                grant_size: self.new_storage_grant_size,
                grant_id: self.new_storage_grant_id.clone(),
            })
            .await?;

        let mut conn = database.acquire().await?;

        ReplicateBlocksTask {
            metadata_id: self.metadata_id.clone(),
            grant_id: self.new_storage_grant_id.clone(),
            block_cids: self.block_cids.clone(),
            new_upload_id: new_upload.upload_id.clone(),
            new_storage_host_url: self.new_host_id.clone(),
            new_storage_host_id: self.new_host_url.clone(),
            old_storage_host_url: self.old_host_id.clone(),
            old_storage_host_id: self.old_host_url.clone(),
        }
        .enqueue::<banyan_task::SqliteTaskStore>(&mut conn)
        .await?;

        Ok(())
    }

    fn unique_key(&self) -> Option<String> {
        Some(self.metadata_id.clone())
    }
}
