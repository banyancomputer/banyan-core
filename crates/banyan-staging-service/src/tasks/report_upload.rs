use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike};
use cid::multibase::Base;
use cid::Cid;
use jwt_simple::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;

use crate::app::AppState;
use crate::clients::core_service::CoreServiceClient;

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
    #[error("http error: {0} response for report upload")]
    HttpError(http::StatusCode),
}

#[derive(Deserialize, Serialize)]
pub struct ReportUploadTask {
    storage_authorization_id: Uuid,
    metadata_id: Uuid,
    cids: Vec<Cid>,
    data_size: u64,
}

impl ReportUploadTask {
    pub fn new(
        storage_authorization_id: Uuid,
        metadata_id: Uuid,
        cids: &[Cid],
        data_size: u64,
    ) -> Self {
        Self {
            storage_authorization_id,
            metadata_id,
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
        let service_signing_key = ctx.secrets().service_signing_key();
        let service_name = ctx.service_name();
        let platform_name = ctx.platform_name();
        let platform_hostname = ctx.platform_hostname();

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
            service_signing_key,
            service_name,
            platform_name,
            platform_hostname,
        );

        let response = client
            .report_upload(
                metadata_id,
                data_size,
                normalized_cids,
                storage_authorization_id,
            )
            .await
            .map_err(ReportUploadTaskError::ReqwestError)?;

        if response.status().is_success() {
            return Ok(());
        } else {
            Err(ReportUploadTaskError::HttpError(response.status()))
        }
    }
}
