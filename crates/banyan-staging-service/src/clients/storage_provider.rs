use cid::Cid;
use http::{HeaderMap, HeaderValue};
use jwt_simple::prelude::*;
use reqwest::multipart::{Form, Part};
use reqwest::{Client, Response};
use serde::Serialize;
use url::Url;

pub struct StorageProviderClient {
    client: Client,
    service_hostname: String,
    service_authorization: String,
}

#[derive(Serialize)]
pub struct ReportUpload {
    data_size: u64,
    normalized_cids: Vec<String>,
    storage_authorization_id: String,
}

#[derive(Deserialize)]
pub struct NewUploadResponse {
    pub upload_id: String,
}

#[derive(Deserialize, Serialize)]
pub struct BlockUploadRequest {
    cid: Cid,
    // Optional additional details about the nature of the upload
    #[serde(flatten)]
    details: BlockUploadDetails,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum BlockUploadDetails {
    Ongoing { completed: bool, upload_id: String },
    OneOff,
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

    pub async fn new_upload(&self, metadata_id: &str) -> Result<NewUploadResponse, reqwest::Error> {
        let full_url = Url::parse(&self.service_hostname)
            .unwrap()
            .join(&"/api/v1/upload/new".to_string())
            .unwrap();

        self.client
            .post(full_url)
            .bearer_auth(&self.service_authorization)
            .body(serde_json::json!({ "metadata_id": metadata_id }).to_string())
            .send()
            .await?
            .json::<NewUploadResponse>()
            .await
    }

    pub async fn upload_block(
        &self,
        block: Vec<u8>,
        cid: Cid,
        details: BlockUploadDetails,
    ) -> Result<Response, reqwest::Error> {
        let full_url = Url::parse(&self.service_hostname)
            .unwrap()
            .join(&"/api/v1/upload/block".to_string())
            .unwrap();

        let block_upload_request = BlockUploadRequest { cid, details };
        let request_json = serde_json::to_string(&block_upload_request).unwrap();

        let request_part = Part::bytes(request_json.as_bytes().to_vec())
            .mime_str("application/json")
            .unwrap();

        let block_part = Part::stream(block)
            .mime_str("application/octet-stream")
            .unwrap();

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
    }
}
