use std::collections::HashSet;

use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike};
use cid::multibase::Base;
use cid::Cid;
use jwt_simple::prelude::*;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use url::Url;
use uuid::Uuid;

use crate::app::AppState;

pub type ReportRedistributionTaskContext = AppState;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum ReportRedistributionTaskError {
    #[error("invalid cid: {0}")]
    InvalidInternalCid(#[from] cid::Error),
    #[error("sql error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("jwt error: {0}")]
    JwtError(#[from] jwt_simple::Error),
    #[error("http error: {0} response from {1}")]
    HttpError(http::StatusCode, Url),
}

#[derive(Deserialize, Serialize)]
pub struct ReportRedistributionTask {
    grant_id: Uuid,
    metadata_id: String,
    cids: Vec<Cid>,
    data_size: u64,
}

impl ReportRedistributionTask {
    pub fn new(grant_id: Uuid, metadata_id: &str, cids: &[Cid], data_size: u64) -> Self {
        Self {
            grant_id,
            metadata_id: String::from(metadata_id),
            cids: cids.to_vec(),
            data_size,
        }
    }
}

#[derive(Serialize)]
struct ReportRedistributionRequest {
    data_size: u64,
    normalized_cids: Vec<String>,
    grant_id: String,
}

#[async_trait]
impl TaskLike for ReportRedistributionTask {
    const TASK_NAME: &'static str = "report_redistribution_task";

    type Error = ReportRedistributionTaskError;
    type Context = ReportRedistributionTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let service_signing_key = ctx.secrets().service_signing_key();
        let service_name = ctx.service_name();
        let platform_name = ctx.platform_name();
        let platform_hostname = ctx.platform_hostname();

        let grant_id = self.grant_id.to_string();
        let data_size = self.data_size;
        let normalized_cids = self
            .cids
            .iter()
            .map(|c| {
                c.to_string_of_base(Base::Base64Url)
                    .map_err(ReportRedistributionTaskError::InvalidInternalCid)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let mut default_headers = HeaderMap::new();
        default_headers.insert("Content-Type", HeaderValue::from_static("application/json"));

        let client = Client::builder()
            .default_headers(default_headers)
            .build()
            .unwrap();

        let report_endpoint = platform_hostname
            .join(&format!(
                "/hooks/storage/redistribution/{}",
                self.metadata_id
            ))
            .unwrap();

        let mut claims = Claims::create(Duration::from_secs(60))
            .with_audiences(HashSet::from_strings(&[platform_name]))
            .with_subject(service_name)
            .invalid_before(Clock::now_since_epoch() - Duration::from_secs(30));

        claims.create_nonce();
        claims.issued_at = Some(Clock::now_since_epoch());

        let bearer_token = service_signing_key.sign(claims).unwrap();

        let report_upload = ReportRedistributionRequest {
            data_size,
            grant_id,
            normalized_cids,
        };

        let request = client
            .post(report_endpoint.clone())
            .json(&report_upload)
            .bearer_auth(bearer_token);

        let response = request
            .send()
            .await
            .map_err(ReportRedistributionTaskError::ReqwestError)?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(ReportRedistributionTaskError::HttpError(
                response.status(),
                report_endpoint,
            ))
        }
    }
}
