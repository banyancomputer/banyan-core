use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike};
use jwt_simple::prelude::*;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::app::AppState;

pub type CompleteUploadBlocksTaskContext = AppState;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum CompleteUploadBlocksTaskError {
    #[error("sql error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("http error: {0} response from {1}")]
    HttpError(http::StatusCode, Url),
}

#[derive(Deserialize, Serialize)]
pub struct CompleteUploadBlocksTask {
    upload_id: String,
    host_id: String,
    url: String,
}

impl CompleteUploadBlocksTask {
    pub fn new(upload_id: String, host_id: String, url: String) -> Self {
        Self {
            upload_id,
            host_id,
            url,
        }
    }
}

#[async_trait]
impl TaskLike for CompleteUploadBlocksTask {
    const TASK_NAME: &'static str = "upload_block_task";

    type Error = CompleteUploadBlocksTaskError;
    type Context = CompleteUploadBlocksTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let mut conn = ctx.database().acquire().await?;
        let service_signing_key = ctx.secrets().service_signing_key();
        let service_name = ctx.service_name();
        let platform_name = ctx.platform_name();
        let platform_hostname = ctx.platform_hostname();

        Ok(())
    }

    fn unique_key(&self) -> Option<String> {
        return Some(self.upload_id.clone());
    }
}
