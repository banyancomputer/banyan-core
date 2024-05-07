use http::{HeaderMap, HeaderValue};
use jwt_simple::prelude::*;
use reqwest::Client;
use url::Url;

use crate::api::DealQuery;
use crate::clients::models::{ApiDeal, ReportUploadRequest};
use crate::clients::{MeterTrafficRequest, ReportRedistributionRequest};
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

    pub async fn get_all_deals(&self, query: DealQuery) -> Result<Vec<ApiDeal>, CoreServiceError> {
        let mut deals_endpoint = self.platform_hostname.join("/api/v1/deals").unwrap();

        if let Some(state) = query.state {
            deals_endpoint = deals_endpoint.join(&format!("?state={}", state)).unwrap();
        }

        let response = self
            .client
            .get(deals_endpoint.clone())
            .bearer_auth(&self.bearer_token)
            .send()
            .await?;

        if response.status().is_success() {
            let deals: Vec<ApiDeal> = response.json().await?;
            return Ok(deals);
        }

        Err(CoreServiceError::BadRequest(response.text().await?))
    }

    pub async fn accept_deal(&self, deal_id: &str) -> Result<(), CoreServiceError> {
        let deal_endpoint = self
            .platform_hostname
            .join(&format!("/api/v1/deals/{}/accept", deal_id))
            .unwrap();

        let response = self
            .client
            .put(deal_endpoint.clone())
            .bearer_auth(&self.bearer_token)
            .send()
            .await?;

        if response.status().is_success() {
            return Ok(());
        }

        Err(CoreServiceError::BadRequest(response.text().await?))
    }

    pub async fn reject_deal(&self, deal_id: &str) -> Result<(), CoreServiceError> {
        let deal_endpoint = self
            .platform_hostname
            .join(&format!("/api/v1/deals/{}/cancel", deal_id))
            .unwrap();

        let response = self
            .client
            .put(deal_endpoint.clone())
            .bearer_auth(&self.bearer_token)
            .send()
            .await?;

        if response.status().is_success() {
            return Ok(());
        }

        Err(CoreServiceError::BadRequest(response.text().await?))
    }

    pub async fn get_deal(&self, deal_id: &str) -> Result<ApiDeal, CoreServiceError> {
        let deal_endpoint = self
            .platform_hostname
            .join(&format!("/api/v1/deals/{}", deal_id))
            .unwrap();

        let response = self
            .client
            .get(deal_endpoint.clone())
            .bearer_auth(&self.bearer_token)
            .send()
            .await?;

        if response.status().is_success() {
            let deal: ApiDeal = response.json().await?;
            return Ok(deal);
        }

        Err(CoreServiceError::BadRequest(response.text().await?))
    }

    pub async fn report_distribution_complete(
        &self,
        metadata_id: &str,
        request: ReportRedistributionRequest,
    ) -> Result<(), CoreServiceError> {
        let storage_hosts_endpoint = self
            .platform_hostname
            .join(&format!("/hooks/storage/distribution/{}", metadata_id))
            .unwrap();

        let response = self
            .client
            .post(storage_hosts_endpoint.clone())
            .json(&request)
            .bearer_auth(&self.bearer_token)
            .send()
            .await?;

        if response.status().is_success() {
            return Ok(());
        }

        Err(CoreServiceError::BadRequest(response.text().await?))
    }

    pub async fn report_upload(
        &self,
        metadata_id: String,
        data_size: u64,
        normalized_cids: Vec<String>,
        storage_authorization_id: String,
    ) -> Result<reqwest::Response, CoreServiceError> {
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
}

#[derive(Debug, thiserror::Error)]
pub enum CoreServiceError {
    #[error("failure during request: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("bad request: {0}")]
    BadRequest(String),
}
