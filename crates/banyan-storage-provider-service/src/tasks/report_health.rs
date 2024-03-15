use async_trait::async_trait;
use banyan_task::{CurrentTask, TaskLike};
use jwt_simple::prelude::*;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use url::Url;

use crate::app::{AppState, Version};

pub type ReportHealthTaskContext = AppState;

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum ReportHealthTaskError {
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("http error: {0} response from {1}")]
    Http(http::StatusCode, Url),
}

#[derive(Deserialize, Serialize, Default)]
pub struct ReportHealthTask;

#[derive(Deserialize, Serialize)]
struct ReportHealth {
    pub build_profile: String,
    pub features: Vec<String>,
    pub version: String,
}

impl From<Version> for ReportHealth {
    fn from(value: Version) -> Self {
        Self {
            build_profile: String::from(value.build_profile),
            features: value
                .features
                .iter()
                .map(|s| String::from(*s))
                .collect::<Vec<_>>(),
            version: String::from(value.version),
        }
    }
}

impl ReportHealth {
    pub fn new() -> Self {
        Version::new().into()
    }
}

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
            .map_err(ReportHealthTaskError::Reqwest)?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(ReportHealthTaskError::Http(
                response.status(),
                report_endpoint,
            ))
        }
    }

    // Schedule every 5 minutes
    fn next_time(&self) -> Option<OffsetDateTime> {
        Some(
            OffsetDateTime::now_utc()
                .checked_add(time::Duration::minutes(5))
                .unwrap(),
        )
    }
}
