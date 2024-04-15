use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use crate::clients::{CoreServiceClient, CoreServiceError};
use crate::utils::is_valid_cid;

pub type ReportUploadTaskContext = AppState;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum ReportUploadTaskError {
    #[error("invalid cid")]
    InvalidCid,
    #[error("sql error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("jwt error: {0}")]
    JwtError(#[from] jwt_simple::Error),
    #[error("core service error: {0}")]
    CoreServiceError(#[from] CoreServiceError),
    #[error("http error: {0}")]
    HttpError(http::StatusCode),
}

#[derive(Deserialize, Serialize)]
pub struct ReportUploadTask {
    storage_authorization_id: Uuid,
    metadata_id: String,
    cids: Vec<String>,
    data_size: u64,
}

impl ReportUploadTask {
    pub fn new(
        storage_authorization_id: Uuid,
        metadata_id: &str,
        cids: &[String],
        data_size: u64,
    ) -> Self {
        Self {
            storage_authorization_id,
            metadata_id: String::from(metadata_id),
            cids: cids.to_vec(),
            data_size,
        }
    }
}

#[async_trait]
impl TaskLike for ReportUploadTask {
    const TASK_NAME: &'static str = "report_upload_task";

    type Error = ReportUploadTaskError;
    type Context = ReportUploadTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let metadata_id = self.metadata_id.to_string();
        let storage_authorization_id = self.storage_authorization_id.to_string();
        let data_size = self.data_size;

        if self.cids.iter().any(|c| !is_valid_cid(c)) {
            return Err(ReportUploadTaskError::InvalidCid);
        }

        let client = CoreServiceClient::new(
            ctx.secrets().service_signing_key(),
            ctx.service_name(),
            ctx.platform_name(),
            ctx.platform_hostname(),
        )?;

        let response = client
            .report_upload(
                metadata_id,
                data_size,
                self.cids.clone(),
                storage_authorization_id,
            )
            .await?;

        if !response.status().is_success() {
            return Err(ReportUploadTaskError::HttpError(response.status()));
        }

        Ok(())
    }
}
