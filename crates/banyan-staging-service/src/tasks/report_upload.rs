use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike};
use cid::multibase::Base;
use cid::Cid;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::AppState;
use crate::clients::{CoreServiceClient, CoreServiceError};

pub type ReportUploadTaskContext = AppState;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum ReportUploadTaskError {
    #[error("invalid cid: {0}")]
    InvalidInternalCid(#[from] cid::Error),
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
    cids: Vec<Cid>,
    data_size: u64,
}

impl ReportUploadTask {
    pub fn new(
        storage_authorization_id: Uuid,
        metadata_id: &str,
        cids: &[Cid],
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
        let normalized_cids = self
            .cids
            .iter()
            .map(|c| {
                c.to_string_of_base(Base::Base64Url)
                    .map_err(ReportUploadTaskError::InvalidInternalCid)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let client = CoreServiceClient::new(
            ctx.secrets().service_signing_key(),
            ctx.service_name(),
            ctx.platform_name(),
            ctx.platform_hostname(),
        );

        let response = client
            .report_upload(
                metadata_id,
                data_size,
                normalized_cids,
                storage_authorization_id,
            )
            .await?;

        if response.status().is_success() {
            return Ok(());
        }

        Err(ReportUploadTaskError::HttpError(response.status()))
    }
}
