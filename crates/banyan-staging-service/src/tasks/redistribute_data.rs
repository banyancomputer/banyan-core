use async_trait::async_trait;
use cid::multibase::Base;
use banyan_task::{CurrentTask, TaskLike};
use jwt_simple::prelude::*;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::app::AppState;
use crate::clients::core_service::CoreServiceClient;
use crate::tasks::report_upload::ReportUploadTaskError;

pub type ReportUploadTaskContext = AppState;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum RedistributeDataTaskError {
    #[error("sql error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("http error: {0} response from {1}")]
    HttpError(http::StatusCode, Url),
}

#[derive(Deserialize, Serialize)]
pub struct RedistributeData {}

impl RedistributeData {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl TaskLike for RedistributeData {
    const TASK_NAME: &'static str = "redistribute_data_task";

    type Error = RedistributeDataTaskError;
    type Context = ReportUploadTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let service_signing_key = ctx.secrets().service_signing_key();
        let service_name = ctx.service_name();
        let platform_name = ctx.platform_name();
        let platform_hostname = ctx.platform_hostname();

        let client = CoreServiceClient::new(
            service_signing_key,
            service_name,
            platform_name,
            platform_hostname,
        );

        let response = client
            .get_storage_providers()
            .await
            .map_err(ReportUploadTaskError::ReqwestError)?;

    }
}
