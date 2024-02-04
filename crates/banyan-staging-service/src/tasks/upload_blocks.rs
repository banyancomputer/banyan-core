use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike};
use jwt_simple::prelude::*;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::app::AppState;
use crate::clients::storage_provider_client::StorageProviderClient;
use crate::database::models::{Blocks, Uploads};

pub type UploadBlocksTaskContext = AppState;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum UploadBlocksTaskError {
    #[error("sql error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("http error: {0} response from {1}")]
    HttpError(http::StatusCode, Url),
    #[error("could not load file {0}")]
    FileLoadError(String),
}

#[derive(Deserialize, Serialize)]
pub struct UploadBlocksTask {
    pub upload_id: String,
    pub storage_host: String,
    pub storage_authorization: String,
}

#[async_trait]
impl TaskLike for UploadBlocksTask {
    const TASK_NAME: &'static str = "upload_block_task";

    type Error = UploadBlocksTaskError;
    type Context = UploadBlocksTaskContext;

    async fn run(&self, task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let service_signing_key = ctx.secrets().service_signing_key();
        let service_name = ctx.service_name();
        let platform_name = ctx.platform_name();
        let platform_hostname = ctx.platform_hostname();
        let database = ctx.database();

        let blocks = Uploads::non_pruned_uploads(&database).await?;

        // let client = StorageProviderClient::new(service_name, platform_hostname);
        //
        // let content = tokio::fs::read(self.file_path.clone())
        //     .await
        //     .map_err(|e| UploadBlocksTaskError::FileLoadError(self.file_path.clone()))?;
        //
        // client.upload_blocks(content, self.url.clone()).await?;

        Ok(())
    }

    fn unique_key(&self) -> Option<String> {
        return Some(self.upload_id.clone());
    }
}
