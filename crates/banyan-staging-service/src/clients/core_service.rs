use std::collections::HashSet;
use futures::task::Spawn;

use http::{HeaderMap, HeaderValue};
use jwt_simple::algorithms::ECDSAP384KeyPairLike;
use jwt_simple::claims::Claims;
use jwt_simple::prelude::*;
use reqwest::{Client, Response};
use serde::Serialize;
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
    pub async fn get_storage_providers(&self) -> Result<ApiStorageHostAdmin, reqwest::Error> {
        let storage_hosts_endpoint = self.platform_hostname.join("/admin/providers").unwrap();

        self.client
            .get(storage_hosts_endpoint.clone())
            .bearer_auth(&self.bearer_token)
            .send()
            .await?
            .json::<ApiStorageHostAdmin>()
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
}