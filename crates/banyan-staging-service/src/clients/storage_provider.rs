use http::{HeaderMap, HeaderValue};
use reqwest::multipart::{Form, Part};
use reqwest::{Client, Response};
use url::Url;

use crate::clients::models::{
    BlockUploadDetailsRequest, BlockUploadRequest, ClientsRequest, NewUploadRequest,
    NewUploadResponse,
};
use crate::clients::{ExistingClientResponse, NewClientResponse};

pub struct StorageProviderClient {
    client: Client,
    service_hostname: String,
    service_authorization: String,
}

impl StorageProviderClient {
    pub fn new(
        service_hostname: &str,
        service_authorization: &str,
    ) -> Result<Self, StorageProviderError> {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_static("application/json"));

        let client = Client::builder().default_headers(headers).build()?;

        Ok(Self {
            client,
            service_hostname: service_hostname.to_string(),
            service_authorization: service_authorization.to_string(),
        })
    }

    pub async fn blocks_present(
        &self,
        block_cids: &[String],
    ) -> Result<Vec<String>, StorageProviderError> {
        let url = Url::parse(&self.service_hostname)
            .map_err(|_| StorageProviderError::UrlParseError)?
            .join("/api/v1/blocks/present")
            .map_err(|_| StorageProviderError::UrlJoinError)?;

        let response = self
            .client
            .get(url)
            .bearer_auth(&self.service_authorization)
            .body(serde_json::json!(block_cids).to_string())
            .send()
            .await
            .map_err(StorageProviderError::RequestError)?;

        if !response.status().is_success() {
            return Err(StorageProviderError::BadRequest(response.text().await?));
        }

        response
            .json::<Vec<String>>()
            .await
            .map_err(|_| StorageProviderError::ResponseParseError)
    }

    pub async fn get_block(&self, cid: &str) -> Result<Vec<u8>, StorageProviderError> {
        let full_url = Url::parse(&self.service_hostname)
            .map_err(|_| StorageProviderError::UrlParseError)?
            .join(&format!("/api/v1/blocks/{}", cid))
            .map_err(|_| StorageProviderError::UrlJoinError)?;

        let response = self
            .client
            .get(full_url)
            .bearer_auth(&self.service_authorization)
            .send()
            .await
            .map_err(StorageProviderError::RequestError)?;

        if !response.status().is_success() {
            return Err(StorageProviderError::BadRequest(response.text().await?));
        }
        Ok(response.bytes().await?.to_vec())
    }

    pub async fn push_client(
        &self,
        client: ClientsRequest,
    ) -> Result<NewClientResponse, StorageProviderError> {
        let full_url = Url::parse(&self.service_hostname)
            .map_err(|_| StorageProviderError::UrlParseError)?
            .join("/api/v1/hooks/clients")
            .map_err(|_| StorageProviderError::UrlJoinError)?;

        let response = self
            .client
            .post(full_url)
            .bearer_auth(&self.service_authorization)
            .body(serde_json::json!(client).to_string())
            .send()
            .await
            .map_err(StorageProviderError::RequestError)?;

        if !response.status().is_success() {
            return Err(StorageProviderError::BadRequest(response.text().await?));
        }
        response
            .json::<NewClientResponse>()
            .await
            .map_err(|_| StorageProviderError::ResponseParseError)
    }

    pub async fn get_client(
        &self,
        metadata_id: &str,
    ) -> Result<ExistingClientResponse, StorageProviderError> {
        let full_url = Url::parse(&self.service_hostname)
            .map_err(|_| StorageProviderError::UrlParseError)?
            .join(&format!("/api/v1/hooks/clients/{}", metadata_id))
            .map_err(|_| StorageProviderError::UrlJoinError)?;

        let response = self
            .client
            .get(full_url)
            .bearer_auth(&self.service_authorization)
            .send()
            .await
            .map_err(StorageProviderError::RequestError)?;

        if !response.status().is_success() {
            return Err(StorageProviderError::BadRequest(response.text().await?));
        }
        response
            .json::<ExistingClientResponse>()
            .await
            .map_err(|_| StorageProviderError::ResponseParseError)
    }

    pub async fn get_upload(
        &self,
        metadata_id: &str,
    ) -> Result<NewUploadResponse, StorageProviderError> {
        let full_url = Url::parse(&self.service_hostname)
            .map_err(|_| StorageProviderError::UrlParseError)?
            .join(&format!("/api/v1/hooks/uploads/{}", metadata_id))
            .map_err(|_| StorageProviderError::UrlJoinError)?;

        let response = self
            .client
            .get(full_url)
            .bearer_auth(&self.service_authorization)
            .send()
            .await
            .map_err(StorageProviderError::RequestError)?;

        if !response.status().is_success() {
            return Err(StorageProviderError::BadRequest(response.text().await?));
        }
        response
            .json::<NewUploadResponse>()
            .await
            .map_err(|_| StorageProviderError::ResponseParseError)
    }

    pub async fn new_upload(
        &self,
        request: &NewUploadRequest,
    ) -> Result<NewUploadResponse, StorageProviderError> {
        let full_url = Url::parse(&self.service_hostname)
            .map_err(|_| StorageProviderError::UrlParseError)?
            .join("/api/v1/hooks/uploads")
            .map_err(|_| StorageProviderError::UrlJoinError)?;

        let response = self
            .client
            .post(full_url)
            .bearer_auth(&self.service_authorization)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(StorageProviderError::BadRequest(response.text().await?));
        }
        response
            .json::<NewUploadResponse>()
            .await
            .map_err(|_| StorageProviderError::ResponseParseError)
    }

    pub async fn upload_block(
        &self,
        block: Vec<u8>,
        cid: String,
        details: BlockUploadDetailsRequest,
    ) -> Result<Response, StorageProviderError> {
        let full_url = Url::parse(&self.service_hostname)
            .map_err(|_| StorageProviderError::UrlParseError)?
            .join("/api/v1/hooks/blocks")
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
