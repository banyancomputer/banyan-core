use async_trait::async_trait;
use banyan_object_store::{ObjectStore, ObjectStoreError, ObjectStorePath};
use banyan_task::{CurrentTask, TaskLike, TaskLikeExt, TaskStoreError};
use jwt_simple::prelude::*;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::app::AppState;
use crate::clients::storage_provider::{BlockUploadDetails, StorageProviderClient};
use crate::database::models::{Blocks, Uploads};
use crate::tasks::complete_upload_blocks::CompleteUploadBlocksTask;

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
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("http error: {0} response from {1}")]
    HttpError(http::StatusCode, Url),
    #[error("could not load file {0}")]
    FileLoadError(String),
    #[error("could not convert object {0} to bytes")]
    ByteConversionError(String),
    #[error("scheduling task error: {0}")]
    SchedulingTaskError(#[from] TaskStoreError),
}

#[derive(Deserialize, Serialize)]
pub struct UploadBlocksTask {
    pub current_upload_id: String,
    pub new_upload_id: String,
    pub storage_host: String,
    pub storage_authorization: String,
}

#[async_trait]
impl TaskLike for UploadBlocksTask {
    const TASK_NAME: &'static str = "upload_block_task";

    type Error = UploadBlocksTaskError;
    type Context = UploadBlocksTaskContext;

    async fn run(&self, task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let mut database = ctx.database();
        let client = StorageProviderClient::new(
            self.storage_host.as_str(),
            self.storage_authorization.as_str(),
        );
        let store = ObjectStore::new(ctx.upload_store_connection())?;

        let upload = match Uploads::get_by_id(&database, &self.current_upload_id).await? {
            Some(uploads) => uploads,
            None => {
                return Err(UploadBlocksTaskError::DatabaseError(
                    sqlx::Error::RowNotFound,
                ))
            }
        };

        // TODO: what's going to happen if the user deletes the file while we redistribute?
        let mut blocks = Blocks::blocks_for_upload(&database, &self.current_upload_id).await?;
        // handling the case where we failed and want to start from another block
        blocks.as_mut_slice().shuffle(&mut rand::thread_rng());

        // TODO: Need to figure our how to handle only failed blocks
        let total_blocks = blocks.len();
        for (index, block) in blocks.into_iter().enumerate() {
            let location =
                ObjectStorePath::from(format!("{}/{}.bin", upload.metadata_id, block.cid));

            let content = store
                .get(&location)
                .await
                .map_err(|e| UploadBlocksTaskError::FileLoadError(location.to_string()))?;
            let content = content
                .bytes()
                .await
                .map_err(|e| UploadBlocksTaskError::ByteConversionError(block.cid.clone()))?;
            let block_cid =
                cid::Cid::try_from(block.cid).map_err(UploadBlocksTaskError::InvalidCid)?;

            let is_last_block = index == total_blocks - 1;
            client
                .upload_block(
                    content.into(),
                    block_cid,
                    BlockUploadDetails::Ongoing {
                        completed: is_last_block,
                        upload_id: self.new_upload_id.clone(),
                    },
                )
                .await?;
        }

        CompleteUploadBlocksTask::new(self.current_upload_id.clone())
            .enqueue::<banyan_task::SqliteTaskStore>(&mut database)
            .await?;

        Ok(())
    }

    fn unique_key(&self) -> Option<String> {
        return Some(self.current_upload_id.clone());
    }
}
