use std::collections::HashSet;

use async_trait::async_trait;
use cid::multibase::Base;
use cid::Cid;
use jwt_simple::prelude::*;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use uuid::Uuid;

use banyan_task::{CurrentTask, TaskLike};

use crate::app::AppState;

pub type ReportUploadTaskContext = AppState;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum ReportUploadTaskError {
    #[error("the task encountered an invalid cid: {0}")]
    InvalidInternalCid(#[from] cid::Error),
    #[error("the task encountered a sql error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("the task encountered a reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("the task encountered a jwt error: {0}")]
    JwtError(#[from] jwt_simple::Error),
    #[error("the task encountered a non success response")]
    NonSuccessResponse(http::StatusCode),
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

#[derive(Serialize)]
struct ReportUpload {
    data_size: u64,
    normalized_cids: Vec<String>,
    storage_authorization_id: String,
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

        let mut default_headers = HeaderMap::new();
        default_headers.insert("Content-Type", HeaderValue::from_static("application/json"));

        let client = Client::builder()
            .default_headers(default_headers)
            .build()
            .unwrap();

        let report_endpoint = platform_hostname
            .join(&format!("/hooks/storage/report/{}", metadata_id))
            .unwrap();

        let mut claims = Claims::create(Duration::from_secs(60))
            .with_audiences(HashSet::from_strings(&[platform_name]))
            .with_subject(service_name)
            .invalid_before(Clock::now_since_epoch() - Duration::from_secs(30));

        claims.create_nonce();
        claims.issued_at = Some(Clock::now_since_epoch());

        let bearer_token = service_signing_key.sign(claims).unwrap();

        let report_upload =
            ReportUpload {
                data_size,
                storage_authorization_id,
                normalized_cids,
            };

        let request = client
            .post(report_endpoint)
            .json(&report_upload)
            .bearer_auth(bearer_token);

        let response = request
            .send()
            .await
            .map_err(ReportUploadTaskError::ReqwestError)?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(ReportUploadTaskError::NonSuccessResponse(response.status()))
        }
    }
}
