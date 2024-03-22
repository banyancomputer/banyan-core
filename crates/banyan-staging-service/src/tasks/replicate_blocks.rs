use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use crate::app::AppState;
use crate::clients::{
    BlockUploadDetailsRequest, CoreServiceClient, CoreServiceError, StorageProviderClient,
    StorageProviderError,
};

pub type ReplicateBlocksTaskContext = AppState;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum ReplicateBlocksTaskError {
    #[error("invalid cid provided in request: {0}")]
    InvalidCid(cid::Error),
    #[error("sql error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("core service error: {0}")]
    CoreServiceError(#[from] CoreServiceError),
    #[error("storage provider error: {0}")]
    StorageProviderError(#[from] StorageProviderError),
}

#[derive(Deserialize, Serialize)]
pub struct ReplicateBlocksTask {
    pub metadata_id: String,
    pub block_cids: Vec<String>,
    pub grant_id: String,
    pub new_upload_id: String,
    pub new_storage_host_url: String,
    pub new_storage_host_id: String,
    pub old_storage_host_url: String,
    pub old_storage_host_id: String,
}

#[async_trait]
impl TaskLike for ReplicateBlocksTask {
    const TASK_NAME: &'static str = "replicate_blocks_task";

    type Error = ReplicateBlocksTaskError;
    type Context = ReplicateBlocksTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let client = CoreServiceClient::new(
            ctx.secrets().service_signing_key(),
            ctx.service_name(),
            ctx.platform_name(),
            ctx.platform_hostname(),
        );
        let provider_credentials = client
            .request_provider_token(&self.old_storage_host_id)
            .await?;
        let old_client =
            StorageProviderClient::new(&self.old_storage_host_url, &provider_credentials.token);

        let provider_credentials = client
            .request_provider_token(&self.new_storage_host_id)
            .await?;
        let new_client =
            StorageProviderClient::new(&self.new_storage_host_url, &provider_credentials.token);

        let mut blocks = self.block_cids.clone();
        // it's possible the below, won't return the block_id even thought it's present on the new host
        // but it seems good enough for now
        let located_blocks = client.locate_blocks(blocks.clone()).await?;
        blocks.retain(|block| {
            if let Some(hosts) = located_blocks.get(block) {
                !hosts.contains(&self.new_storage_host_url)
            } else {
                true
            }
        });

        // handling the case where we failed and want to start from another block
        // so that in the end only the failing block would be left
        blocks.as_mut_slice().shuffle(&mut rand::thread_rng());
        let total_blocks = blocks.len();
        for (index, block_cid) in blocks.into_iter().enumerate() {
            let fetched_block = old_client.get_block(&block_cid).await?;
            let block_cid =
                cid::Cid::try_from(block_cid).map_err(ReplicateBlocksTaskError::InvalidCid)?;

            let is_last_block = index == total_blocks - 1;
            new_client
                .upload_block(
                    fetched_block,
                    block_cid,
                    BlockUploadDetailsRequest {
                        completed: is_last_block,
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

impl ReplicateBlocksTask {
    pub fn new_with_metadata_id(metadata_id: String) -> Self {
        Self {
            metadata_id,
            block_cids: Vec::new(),
            grant_id: String::new(),
            new_upload_id: String::new(),
            new_storage_host_id: String::new(),
            new_storage_host_url: String::new(),
            old_storage_host_id: String::new(),
            old_storage_host_url: String::new(),
        }
    }
}
