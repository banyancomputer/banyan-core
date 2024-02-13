use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike};
use jwt_simple::prelude::*;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use url::Url;

use crate::app::{AppState, Version};

pub type ReportHealthTaskContext = AppState;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum ReportHealthTaskError {
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
pub struct ReportHealthTask;
type ReportHealth = Version;

#[async_trait]
impl TaskLike for ReportHealthTask {
    const TASK_NAME: &'static str = "report_health_task";

    type Error = ReportHealthTaskError;
    type Context = ReportHealthTaskContext;

    async fn run(&self, _task: CurrentTask, ctx: Self::Context) -> Result<(), Self::Error> {
        let service_signing_key = ctx.secrets().service_signing_key();
        let service_name = ctx.service_name();
        let platform_name = ctx.platform_name();
        let platform_hostname = ctx.platform_hostname();

        let mut default_headers = HeaderMap::new();
        default_headers.insert("Content-Type", HeaderValue::from_static("application/json"));

        let client = Client::builder()
            .default_headers(default_headers)
            .build()
            .unwrap();

        let report_endpoint = platform_hostname
            .join("/hooks/storage/report/health")
            .unwrap();

        let mut claims = Claims::create(Duration::from_secs(60))
            .with_audiences(HashSet::from_strings(&[platform_name]))
            .with_subject(service_name)
            .invalid_before(Clock::now_since_epoch() - Duration::from_secs(30));

        claims.create_nonce();
        claims.issued_at = Some(Clock::now_since_epoch());

        let bearer_token = service_signing_key.sign(claims).unwrap();

        let request = client
            .post(report_endpoint.clone())
            .json(&ReportHealth::new())
            .bearer_auth(bearer_token);

        let response = request
            .send()
            .await
            .map_err(ReportHealthTaskError::ReqwestError)?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(ReportHealthTaskError::HttpError(
                response.status(),
                report_endpoint,
            ))
        }
    }
}
