use cid::Cid;
use http::{HeaderMap, HeaderValue};
use reqwest::multipart::{Form, Part};
use reqwest::{Client, Response};
use url::Url;

use crate::clients::models::{BlockUploadDetailsRequest, BlockUploadRequest, NewUploadResponse};
use crate::database::models::Clients;

pub struct StorageProviderClient {
    client: Client,
    service_hostname: String,
    service_authorization: String,
}

impl StorageProviderClient {
    pub fn new(service_hostname: &str, service_authorization: &str) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));

        let client = Client::builder().default_headers(headers).build().unwrap();

        Self {
            client,
            service_hostname: service_hostname.to_string(),
            service_authorization: service_authorization.to_string(),
        }
    }

    pub async fn push_client(&self, client: Clients) -> Result<(), StorageProviderError> {
        let full_url = Url::parse(&self.service_hostname)
            .map_err(|_| StorageProviderError::UrlParseError)?
            .join(&"/api/v1/admin/clients".to_string())
            .map_err(|_| StorageProviderError::UrlJoinError)?;

        let response = self
            .client
            .post(full_url)
            .bearer_auth(&self.service_authorization)
            .body(serde_json::json!(client).to_string())
            .send()
            .await
            .map_err(StorageProviderError::RequestError)?;

        if response.status().is_success() {
            return Ok(());
        }

        Err(StorageProviderError::BadRequest(response.text().await?))
    }
    pub async fn new_upload(
        &self,
        metadata_id: &str,
    ) -> Result<NewUploadResponse, StorageProviderError> {
        let full_url = Url::parse(&self.service_hostname)
            .map_err(|_| StorageProviderError::UrlParseError)?
            .join(&"/api/v1/upload/new".to_string())
            .map_err(|_| StorageProviderError::UrlJoinError)?;

        let response = self
            .client
            .post(full_url)
            .bearer_auth(&self.service_authorization)
            .body(serde_json::json!({ "metadata_id": metadata_id }).to_string())
            .send()
            .await
            .map_err(StorageProviderError::RequestError)?;

        if response.status().is_success() {
            return match response.json::<NewUploadResponse>().await {
                Ok(response) => Ok(response),
                Err(_) => Err(StorageProviderError::ResponseParseError),
            };
        }

        Err(StorageProviderError::BadRequest(response.text().await?))
    }

    pub async fn upload_block(
        &self,
        block: Vec<u8>,
        cid: Cid,
        details: BlockUploadDetailsRequest,
    ) -> Result<Response, StorageProviderError> {
        let full_url = Url::parse(&self.service_hostname)
            .map_err(|_| StorageProviderError::UrlParseError)?
            .join(&"/api/v1/upload/block".to_string())
            .map_err(|_| StorageProviderError::UrlJoinError)?;

        let block_upload_request = BlockUploadRequest { cid, details };
        let request_json = serde_json::to_string(&block_upload_request).unwrap();

        let request_part = Part::bytes(request_json.as_bytes().to_vec())
            .mime_str("application/json")
            .map_err(|_| StorageProviderError::MimeStrError)?;

        let block_part = Part::stream(block)
            .mime_str("application/octet-stream")
            .map_err(|_| StorageProviderError::MimeStrError)?;

        let multipart_form = Form::new()
            .part("request-data", request_part)
            .part("block", block_part);

        // post
        self.client
            .post(full_url)
            .bearer_auth(&self.service_authorization)
            .multipart(multipart_form)
            .send()
            .await
            .map_err(StorageProviderError::RequestError)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StorageProviderError {
    #[error("failure during request: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("url parse error")]
    UrlParseError,
    #[error("url join error")]
    UrlJoinError,
    #[error("mime str error")]
    MimeStrError,
    #[error("response parse error")]
    ResponseParseError,
}
