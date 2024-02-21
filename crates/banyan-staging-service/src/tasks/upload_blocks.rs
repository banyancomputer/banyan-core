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
use crate::database::models::Blocks;

pub type UploadBlocksTaskContext = AppState;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum UploadBlocksTaskError {
    #[error("invalid cid provided in request: {0}")]
    InvalidCid(cid::Error),
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
pub struct UploadBlocksTask {
    pub current_upload_id: String,
    pub metadata_id: String,
    pub grant_id: String,
    pub new_upload_id: String,
    pub storage_host_url: String,
    pub storage_host_id: String,
}

#[async_trait]
impl TaskLike for UploadBlocksTask {
    const TASK_NAME: &'static str = "upload_block_task";

    type Error = UploadBlocksTaskError;
    type Context = UploadBlocksTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let database = ctx.database();
        let client = CoreServiceClient::new(
            ctx.secrets().service_signing_key(),
            ctx.service_name(),
            ctx.platform_name(),
            ctx.platform_hostname(),
        );
        let provider_credentials = client.request_provider_token(&self.storage_host_id).await?;

        let client =
            StorageProviderClient::new(&self.storage_host_url, &provider_credentials.token);

        let mut blocks = Blocks::blocks_for_upload(&database, &self.current_upload_id).await?;
        // handling the case where we failed and want to start from another block
        // so that in the end only the failing block would be left
        blocks.as_mut_slice().shuffle(&mut rand::thread_rng());
        let total_blocks = blocks.len();

        let store = ObjectStore::new(ctx.upload_store_connection())?;
        for (index, block) in blocks.into_iter().enumerate() {
            let location =
                ObjectStorePath::from(format!("{}/{}.bin", &self.metadata_id, block.cid));

            let content = store
                .get(&location)
                .await
                .map_err(|_| UploadBlocksTaskError::FileLoadError(location.to_string()))?;
            let content = content
                .bytes()
                .await
                .map_err(|_| UploadBlocksTaskError::ByteConversionError(block.cid.clone()))?;
            let block_cid =
                cid::Cid::try_from(block.cid).map_err(UploadBlocksTaskError::InvalidCid)?;

            let is_last_block = index == total_blocks - 1;
            client
                .upload_block(
                    content.into(),
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

impl UploadBlocksTask {
    pub fn new_with_metadata_id(metadata_id: String) -> Self {
        Self {
            metadata_id,
            current_upload_id: String::new(),
            grant_id: String::new(),
            new_upload_id: String::new(),
            storage_host_url: String::new(),
            storage_host_id: String::new(),
        }
    }
}
