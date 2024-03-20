use http::{HeaderMap, HeaderValue};
use jwt_simple::prelude::*;
use reqwest::{Client, Response};
use url::Url;

use crate::clients::models::{ReportUploadRequest, StorageProviderAuthResponse};
use crate::clients::MeterTrafficRequest;
use crate::utils::SigningKey;

pub struct CoreServiceClient {
    client: Client,
    bearer_token: String,
    platform_hostname: Url,
}

impl CoreServiceClient {
    pub fn new(
        service_signing_key: SigningKey,
        service_name: &str,
        platform_name: &str,
        platform_hostname: Url,
    ) -> Self {
        let mut claims = Claims::create(Duration::from_secs(60))
            .with_audiences(HashSet::from_strings(&[platform_name]))
            .with_subject(service_name)
            .invalid_before(Clock::now_since_epoch() - Duration::from_secs(30));

        claims.create_nonce();
        claims.issued_at = Some(Clock::now_since_epoch());
        let bearer_token = service_signing_key.sign(claims).unwrap();
        let mut default_headers = HeaderMap::new();
        default_headers.insert("Content-Type", HeaderValue::from_static("application/json"));

        let client = Client::builder()
            .default_headers(default_headers)
            .build()
            .unwrap();

        Self {
            client,
            bearer_token,
            platform_hostname,
        }
    }

    pub async fn report_upload(
        &self,
        metadata_id: String,
        data_size: u64,
        normalized_cids: Vec<String>,
        storage_authorization_id: String,
    ) -> Result<Response, CoreServiceError> {
        let report_upload = ReportUploadRequest {
            data_size,
            storage_authorization_id,
            normalized_cids,
        };

        let report_endpoint = self
            .platform_hostname
            .join(&format!("/hooks/storage/report/{}", metadata_id))
            .unwrap();

        let response = self
            .client
            .post(report_endpoint.clone())
            .json(&report_upload)
            .bearer_auth(&self.bearer_token)
            .send()
            .await?;

        if response.status().is_success() {
            return Ok(response);
        }

        Err(CoreServiceError::BadRequest(response.text().await?))
    }

    pub async fn request_provider_token(
        &self,
        storage_provider_id: &str,
    ) -> Result<StorageProviderAuthResponse, CoreServiceError> {
        let provider_token_url = self
            .platform_hostname
            .join(format!("/api/v1/auth/provider_grant/{}", storage_provider_id).as_str())
            .unwrap();

        let response = self
            .client
            .get(provider_token_url.clone())
            .bearer_auth(&self.bearer_token)
            .send()
            .await?;

        if response.status().is_success() {
            return match response.json::<StorageProviderAuthResponse>().await {
                Ok(response) => Ok(response),
                Err(_) => Err(CoreServiceError::ResponseParseError),
            };
        }

        Err(CoreServiceError::BadRequest(response.text().await?))
    }

    pub async fn report_user_bandwidth(
        &self,
        traffic_metrics: MeterTrafficRequest<'_>,
    ) -> Result<(), CoreServiceError> {
        let storage_hosts_endpoint = self
            .platform_hostname
            .join("/api/v1/metrics/traffic")
            .unwrap();

        let response = self
            .client
            .post(storage_hosts_endpoint.clone())
            .json(&traffic_metrics)
            .bearer_auth(&self.bearer_token)
            .send()
            .await?;

        if response.status().is_success() {
            return Ok(());
        }

        Err(CoreServiceError::BadRequest(response.text().await?))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CoreServiceError {
    #[error("response parse error")]
    ResponseParseError,
    #[error("failure during request: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("bad request: {0}")]
    BadRequest(String),
}
