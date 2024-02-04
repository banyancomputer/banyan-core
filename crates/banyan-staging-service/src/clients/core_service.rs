use std::collections::HashSet;

use http::{HeaderMap, HeaderValue};
use jwt_simple::algorithms::ECDSAP384KeyPairLike;
use jwt_simple::claims::Claims;
use jwt_simple::prelude::*;
use reqwest::{Client, Response};
use serde::Serialize;
use tracing::error_span;
use url::Url;

use crate::clients::models::ApiStorageHostAdmin;
use crate::utils::SigningKey;

pub struct CoreServiceClient {
    client: Client,
    bearer_token: String,
    platform_hostname: Url,
}

#[derive(Serialize)]
pub struct ReportUpload {
    data_size: u64,
    normalized_cids: Vec<String>,
    storage_authorization_id: String,
}

#[derive(Serialize)]
pub struct MoveMetadataRequest {
    pub needed_capacity: i64,
    // block IDs stored on the old host, to be deleted and then moved to a new host
    pub previous_cids: Vec<String>,
}

#[derive(Deserialize)]
pub struct MoveMetadataResponse {
    pub storage_host: String,
    pub storage_authorization: String,
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
    pub async fn get_storage_providers(&self) -> Result<Vec<ApiStorageHostAdmin>, reqwest::Error> {
        let storage_hosts_endpoint = self.platform_hostname.join("/admin/providers").unwrap();

        self.client
            .get(storage_hosts_endpoint.clone())
            .bearer_auth(&self.bearer_token)
            .send()
            .await?
            .json::<Vec<ApiStorageHostAdmin>>()
            .await
    }

    pub async fn report_upload(
        &self,
        metadata_id: String,
        data_size: u64,
        normalized_cids: Vec<String>,
        storage_authorization_id: String,
    ) -> Result<Response, reqwest::Error> {
        let report_upload = ReportUpload {
            data_size,
            storage_authorization_id,
            normalized_cids,
        };

        let report_endpoint = self
            .platform_hostname
            .join(&format!("/hooks/storage/report/{}", metadata_id))
            .unwrap();

        self.client
            .post(report_endpoint.clone())
            .json(&report_upload)
            .bearer_auth(&self.bearer_token)
            .send()
            .await
    }

    pub async fn initiate_metadata_move(
        &self,
        metadata_id: &String,
        request: MoveMetadataRequest,
    ) -> Result<MoveMetadataResponse, reqwest::Error> {
        let token_endpoint = self
            .platform_hostname
            .join(format!("/api/v1/bucket/metadata/{}/move", metadata_id).as_str())
            .unwrap();
        let response = self
            .client
            .post(token_endpoint)
            .header("Content-Type", "application/json")
            .json(&request)
            .bearer_auth(&self.bearer_token)
            .send()
            .await?;

        response.json::<MoveMetadataResponse>().await
    }
}
