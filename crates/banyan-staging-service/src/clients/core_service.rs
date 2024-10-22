use std::collections::HashMap;

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
    ) -> Result<Self, CoreServiceError> {
        let mut claims = Claims::create(Duration::from_secs(60))
            .with_audiences(HashSet::from_strings(&[platform_name]))
            .with_subject(service_name)
            .invalid_before(Clock::now_since_epoch() - Duration::from_secs(30));

        claims.create_nonce();
        claims.issued_at = Some(Clock::now_since_epoch());
        let bearer_token = service_signing_key
            .sign(claims)
            .map_err(|_| CoreServiceError::TokenSigningError)?;
        let mut default_headers = HeaderMap::new();
        default_headers.insert("Content-Type", HeaderValue::from_static("application/json"));

        let client = Client::builder()
            .default_headers(default_headers)
            .build()
            .map_err(|_| CoreServiceError::ClientBuildingError)?;

        Ok(Self {
            client,
            bearer_token,
            platform_hostname,
        })
    }
    pub async fn locate_blocks(
        &self,
        block_cids: Vec<String>,
    ) -> Result<HashMap<String, Vec<String>>, CoreServiceError> {
        let locate_blocks_endpoint = self
            .platform_hostname
            .join("/api/v1/blocks/locate")
            .map_err(|_| CoreServiceError::UrlJoinError)?;

        let response = self
            .client
            .post(locate_blocks_endpoint.clone())
            .json(&block_cids)
            .bearer_auth(&self.bearer_token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(CoreServiceError::BadRequest(response.text().await?));
        }
        response
            .json::<HashMap<String, Vec<String>>>()
            .await
            .map_err(|_| CoreServiceError::ResponseParseError)
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
            .map_err(|_| CoreServiceError::UrlJoinError)?;

        let response = self
            .client
            .post(report_endpoint.clone())
            .json(&report_upload)
            .bearer_auth(&self.bearer_token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(CoreServiceError::BadRequest(response.text().await?));
        }
        Ok(response)
    }

    pub async fn request_provider_token(
        &self,
        storage_provider_id: &str,
    ) -> Result<StorageProviderAuthResponse, CoreServiceError> {
        let provider_token_url = self
            .platform_hostname
            .join(format!("/api/v1/auth/provider_grant/{}", storage_provider_id).as_str())
            .map_err(|_| CoreServiceError::UrlJoinError)?;

        let response = self
            .client
            .get(provider_token_url.clone())
            .bearer_auth(&self.bearer_token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(CoreServiceError::BadRequest(response.text().await?));
        }
        response
            .json::<StorageProviderAuthResponse>()
            .await
            .map_err(|_| CoreServiceError::ResponseParseError)
    }

    pub async fn report_user_bandwidth(
        &self,
        traffic_metrics: MeterTrafficRequest<'_>,
    ) -> Result<(), CoreServiceError> {
        let storage_hosts_endpoint = self
            .platform_hostname
            .join("/api/v1/metrics/traffic")
            .map_err(|_| CoreServiceError::UrlJoinError)?;

        let response = self
            .client
            .post(storage_hosts_endpoint.clone())
            .json(&traffic_metrics)
            .bearer_auth(&self.bearer_token)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(CoreServiceError::BadRequest(response.text().await?));
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CoreServiceError {
    #[error("client building error")]
    ClientBuildingError,
    #[error("token signing error")]
    TokenSigningError,
    #[error("url join error")]
    UrlJoinError,
    #[error("response parse error")]
    ResponseParseError,
    #[error("failure during request: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("bad request: {0}")]
    BadRequest(String),
}
