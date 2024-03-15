use http::{HeaderMap, HeaderValue};
use jwt_simple::prelude::*;
use reqwest::Client;
use serde::Serialize;
use url::Url;

use crate::app::ServiceKey;
use crate::clients::{DeleteBlocksRequest, DistributeDataRequest, ReplicateDataRequest};

pub struct StagingServiceClient {
    client: Client,
    bearer_token: String,
    staging_service_hostname: Url,
}

#[derive(Serialize)]
pub struct ReportUpload {
    data_size: u64,
    normalized_cids: Vec<String>,
    storage_authorization_id: String,
}
impl StagingServiceClient {
    pub fn new(
        service_signing_key: ServiceKey,
        service_name: &str,
        staging_service_name: &str,
        staging_service_hostname: Url,
    ) -> Self {
        let mut claims = Claims::create(Duration::from_secs(60))
            .with_audiences(HashSet::from_strings(&[staging_service_name]))
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
            staging_service_hostname,
        }
    }

    pub async fn delete_blocks(
        &self,
        request: DeleteBlocksRequest,
    ) -> Result<(), StagingServiceError> {
        let endpoint = self
            .staging_service_hostname
            .join("/api/v1/hooks/blocks")
            .unwrap();

        let response = self
            .client
            .delete(endpoint)
            .json(&request)
            .bearer_auth(&self.bearer_token)
            .send()
            .await?;

        if response.status().is_success() {
            return Ok(());
        }

        Err(StagingServiceError::BadRequest(response.text().await?))
    }

    pub async fn distribute_data(
        &self,
        request: DistributeDataRequest,
    ) -> Result<(), StagingServiceError> {
        let endpoint = self
            .staging_service_hostname
            .join("/api/v1/hooks/distribute")
            .unwrap();

        let response = self
            .client
            .post(endpoint)
            .json(&request)
            .bearer_auth(&self.bearer_token)
            .send()
            .await?;

        if response.status().is_success() {
            return Ok(());
        }

        Err(StagingServiceError::BadRequest(response.text().await?))
    }
    pub async fn replicate_data(
        &self,
        request: ReplicateDataRequest,
    ) -> Result<(), StagingServiceError> {
        let endpoint = self
            .staging_service_hostname
            .join("/api/v1/hooks/replicate")
            .unwrap();

        let response = self
            .client
            .post(endpoint)
            .json(&request)
            .bearer_auth(&self.bearer_token)
            .send()
            .await?;

        if response.status().is_success() {
            return Ok(());
        }

        Err(StagingServiceError::BadRequest(response.text().await?))
    }


}

#[derive(Debug, thiserror::Error)]
pub enum StagingServiceError {
    #[error("failure during request: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("jwt error: {0}")]
    JwtError(#[from] jwt_simple::Error),
}
