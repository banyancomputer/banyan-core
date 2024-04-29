use std::collections::HashSet;

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
    #[error("invalid cid provided in request")]
    InvalidCid,
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
        )?;
        let provider_credentials = client
            .request_provider_token(&self.old_storage_host_id)
            .await?;
        let old_client =
            StorageProviderClient::new(&self.old_storage_host_url, &provider_credentials.token)?;

        let provider_credentials = client
            .request_provider_token(&self.new_storage_host_id)
            .await?;
        let new_client =
            StorageProviderClient::new(&self.new_storage_host_url, &provider_credentials.token)?;

        let replicated_blocks = new_client
            .blocks_present(self.block_cids.as_slice())
            .await?;
        let replicated_blocks_set: HashSet<_> = replicated_blocks.into_iter().collect();
        let mut non_replicated: Vec<_> = self
            .block_cids
            .iter()
            .filter(|block_cid| !replicated_blocks_set.contains(*block_cid))
            .cloned()
            .collect();

        // handling the case where we failed and want to start from another block
        // so that in the end only the failing block would be left
        non_replicated
            .as_mut_slice()
            .shuffle(&mut rand::thread_rng());
        let mut blocks_iter = non_replicated.into_iter().peekable();
        while let Some(block_cid) = blocks_iter.next() {
            let fetched_block = old_client.get_block(&block_cid).await?;

            new_client
                .upload_block(
                    fetched_block,
                    block_cid,
                    BlockUploadDetailsRequest {
                        replication: true,
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
        Some(format!("{}-{}", self.new_storage_host_id, self.metadata_id))
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
