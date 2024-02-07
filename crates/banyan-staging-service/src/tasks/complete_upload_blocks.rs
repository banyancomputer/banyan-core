use async_trait::async_trait;
use banyan_object_store::{ObjectStore, ObjectStoreError};
use banyan_task::{CurrentTask, TaskLike};
use jwt_simple::prelude::*;
use serde::{Deserialize, Serialize};

use crate::app::AppState;
use crate::clients::core_service::{CoreServiceClient, CoreServiceError};
use crate::database::models::{Blocks, Uploads, UploadsBlocks};

pub type CompleteUploadBlocksTaskContext = AppState;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum CompleteUploadBlocksTaskError {
    #[error("sql error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("object store error: {0}")]
    ObjectStoreError(#[from] ObjectStoreError),
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] CoreServiceError),
    #[error("not all blocks uploaded to remote storage: {0}/{1}")]
    NotAllBlocksUploaded(usize, usize),
}

#[derive(Deserialize, Serialize)]
pub struct CompleteUploadBlocksTask {
    pub upload_id: String,
}

impl CompleteUploadBlocksTask {
    pub fn new(upload_id: String) -> Self {
        Self { upload_id }
    }
}

#[async_trait]
impl TaskLike for CompleteUploadBlocksTask {
    const TASK_NAME: &'static str = "complete_upload_blocks_task";

    type Error = CompleteUploadBlocksTaskError;
    type Context = CompleteUploadBlocksTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let conn = ctx.database();

        let store = ObjectStore::new(ctx.upload_store_connection())?;
        let core_client = CoreServiceClient::new(
            ctx.secrets().service_signing_key(),
            ctx.service_name(),
            ctx.platform_name(),
            ctx.platform_hostname(),
        );

        // TODO: what's going to happen if the user deletes the file while we redistribute?
        let blocks_for_upload = Blocks::blocks_for_upload(&conn, &self.upload_id).await?;
        let block_cids: Vec<String> = blocks_for_upload
            .iter()
            .map(|block| block.cid.clone())
            .collect();

        let locate_blocks_response = core_client.locate_blocks(block_cids.clone()).await?;

        let self_hostname = ctx.service_hostname().to_string();
        let blocks_not_associated_with_us: HashSet<String> = locate_blocks_response
            .iter()
            .filter_map(|(host, block_cids)| {
                if !host.contains(&self_hostname) {
                    return Some(block_cids.clone());
                }
                return None;
            })
            .flatten()
            .collect();

        if blocks_not_associated_with_us.len() < block_cids.len() {
            return Err(CompleteUploadBlocksTaskError::NotAllBlocksUploaded(
                blocks_not_associated_with_us.len(),
                block_cids.len(),
            ));
        }
        let metadata_id = match Uploads::get_by_id(&conn, &self.upload_id).await? {
            Some(upload) => upload.metadata_id,
            None => {
                return Err(CompleteUploadBlocksTaskError::DatabaseError(
                    sqlx::Error::RowNotFound,
                ))
            }
        };
        for block_id in block_cids.iter() {
            UploadsBlocks::mark_as_pruned(&conn, &block_id).await?;
            let location = banyan_object_store::ObjectStorePath::from(format!(
                "{}/{}.bin",
                metadata_id, block_id
            ));
            store.delete(&location).await?;
        }

        Ok(())
    }

    fn unique_key(&self) -> Option<String> {
        return Some(self.upload_id.clone());
    }
}
