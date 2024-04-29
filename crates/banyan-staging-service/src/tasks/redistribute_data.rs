use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike, TaskLikeExt, TaskStoreError};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::app::AppState;
use crate::clients::{
    ClientsRequest, CoreServiceClient, CoreServiceError, NewUploadRequest, StorageProviderClient,
    StorageProviderError,
};
use crate::database::models::{Clients, Uploads};
use crate::tasks::RedistributeBlocksTask;

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
    pub metadata_id: String,
    pub storage_grant_id: String,
    pub storage_grant_size: i64,
    pub block_cids: Vec<String>,
    pub new_host_id: String,
    pub new_host_url: String,
}

#[async_trait]
impl TaskLike for RedistributeDataTask {
    const TASK_NAME: &'static str = "redistribute_data_task";

    type Error = RedistributeDataTaskError;
    type Context = RedistributeDataTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let database = ctx.database();
        let client = CoreServiceClient::new(
            ctx.secrets().service_signing_key(),
            ctx.service_name(),
            ctx.platform_name(),
            ctx.platform_hostname(),
        )?;
        let provider_credentials = client.request_provider_token(&self.new_host_id).await?;
        let storage_client =
            StorageProviderClient::new(&self.new_host_url, &provider_credentials.token)?;
        let upload = Uploads::get_by_metadata_id(&database, &self.metadata_id).await?;

        let client = Clients::find_by_upload_id(&database, &upload.id).await?;
        let new_client = storage_client
            .push_client(ClientsRequest {
                platform_id: client.platform_id,
                fingerprint: client.fingerprint,
                public_key: client.public_key,
            })
            .await?;

        let new_upload = storage_client
            .new_upload(&NewUploadRequest {
                metadata_id: upload.metadata_id,
                client_id: new_client.id.clone(),
                grant_id: self.storage_grant_id.clone(),
                grant_size: self.storage_grant_size,
            })
            .await?;

        let mut conn = database.acquire().await?;

        RedistributeBlocksTask {
            metadata_id: self.metadata_id.clone(),
            grant_id: self.storage_grant_id.clone(),
            block_cids: self.block_cids.clone(),
            new_upload_id: new_upload.upload_id.clone(),
            storage_host_id: self.new_host_id.clone(),
            storage_host_url: self.new_host_url.clone(),
        }
        .enqueue::<banyan_task::SqliteTaskStore>(&mut conn)
        .await?;

        Ok(())
    }

    fn unique_key(&self) -> Option<String> {
        Some(self.metadata_id.clone())
    }
}
