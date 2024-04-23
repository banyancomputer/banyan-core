use async_trait::async_trait;
use banyan_object_store::{ObjectStore, ObjectStoreError, ObjectStorePath};
use banyan_task::{CurrentTask, TaskLike, TaskStoreError};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use crate::app::AppState;
use crate::clients::{
    BlockUploadDetailsRequest, CoreServiceClient, CoreServiceError, StorageProviderClient,
    StorageProviderError,
};
use crate::utils::is_valid_cid;

pub type RedistributeBlocksTaskContext = AppState;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum RedistributeBlocksTaskError {
    #[error("invalid cid provided in request")]
    InvalidCid,
    #[error("object store error: {0}")]
    ObjectStoreError(#[from] ObjectStoreError),
    #[error("sql error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("core service error: {0}")]
    CoreServiceError(#[from] CoreServiceError),
    #[error("could not load file {0}")]
    FileLoadError(String),
    #[error("could not convert object {0} to bytes")]
    ByteConversionError(String),
    #[error("scheduling task error: {0}")]
    SchedulingTaskError(#[from] TaskStoreError),
    #[error("storage provider error: {0}")]
    StorageProviderError(#[from] StorageProviderError),
}

#[derive(Deserialize, Serialize)]
pub struct RedistributeBlocksTask {
    pub metadata_id: String,
    pub block_cids: Vec<String>,
    pub grant_id: String,
    pub new_upload_id: String,
    pub storage_host_url: String,
    pub storage_host_id: String,
}

#[async_trait]
impl TaskLike for RedistributeBlocksTask {
    const TASK_NAME: &'static str = "redistribute_blocks_task";

    type Error = RedistributeBlocksTaskError;
    type Context = RedistributeBlocksTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let client = CoreServiceClient::new(
            ctx.secrets().service_signing_key(),
            ctx.service_name(),
            ctx.platform_name(),
            ctx.platform_hostname(),
        )?;
        let provider_credentials = client.request_provider_token(&self.storage_host_id).await?;
        let client =
            StorageProviderClient::new(&self.storage_host_url, &provider_credentials.token);

        let mut blocks = self.block_cids.clone();
        if blocks.iter().any(|c| !is_valid_cid(c)) {
            return Err(RedistributeBlocksTaskError::InvalidCid);
        }

        // handling the case where we failed and want to start from another block
        // so that in the end only the failing block would be left
        blocks.as_mut_slice().shuffle(&mut rand::thread_rng());
        let store = ObjectStore::new(ctx.upload_store_connection())?;
        let mut blocks_iter = blocks.into_iter().peekable();
        while let Some(block_cid) = blocks_iter.next() {
            let location =
                ObjectStorePath::from(format!("{}/{}.bin", &self.metadata_id, block_cid));

            let content = store
                .get(&location)
                .await
                .map_err(|_| RedistributeBlocksTaskError::FileLoadError(location.to_string()))?;
            let content = content
                .bytes()
                .await
                .map_err(|_| RedistributeBlocksTaskError::ByteConversionError(block_cid.clone()))?;

            client
                .upload_block(
                    content.into(),
                    block_cid,
                    BlockUploadDetailsRequest {
                        replication: false,
                        completed: blocks_iter.peek().is_none(),
                        grant_id: self.grant_id.clone(),
                        upload_id: self.new_upload_id.clone(),
                    },
                )
                .await?;
        }

        Ok(())
    }

    fn unique_key(&self) -> Option<String> {
        Some(self.metadata_id.clone())
    }
}

impl RedistributeBlocksTask {
    pub fn new_with_metadata_id(metadata_id: String) -> Self {
        Self {
            metadata_id,
            block_cids: Vec::new(),
            grant_id: String::new(),
            new_upload_id: String::new(),
            storage_host_url: String::new(),
            storage_host_id: String::new(),
        }
    }
}
