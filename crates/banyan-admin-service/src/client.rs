use banyan_api_client::prelude::{ApiRequest, InfallibleError};
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::models::{ApiDeals, ApiStorageHost};

#[derive(Serialize, Deserialize)]
pub struct CreateStorageProvidersRequest {
    pub name: String,
    pub url: String,
    pub used_storage: i64,
    pub available_storage: i64,
    pub fingerprint: String,
    pub pem: String,
}

impl ApiRequest for CreateStorageProvidersRequest {
    type ResponseType = ApiStorageHost;
    type ErrorType = InfallibleError;

    fn build_request(self, base_url: &Url, client: &Client) -> RequestBuilder {
        let full_url = base_url.join("/api/v1/admin/providers").unwrap();
        client.post(full_url).json(&self)
    }

    fn requires_authentication(&self) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct GetAllStorageProvidersRequest;

impl ApiRequest for GetAllStorageProvidersRequest {
    type ResponseType = ApiStorageHost;
    type ErrorType = InfallibleError;

    fn build_request(self, base_url: &Url, client: &Client) -> RequestBuilder {
        let full_url = base_url.join("/api/v1/admin/providers").unwrap();
        client.get(full_url)
    }

    fn requires_authentication(&self) -> bool {
        true
    }
}

#[derive(Debug)]
pub struct GetAllDealsRequest;

impl ApiRequest for GetAllDealsRequest {
    type ResponseType = ApiDeals;
    type ErrorType = InfallibleError;

    fn build_request(self, base_url: &Url, client: &Client) -> RequestBuilder {
        let full_url = base_url.join("/api/v1/admin/deals").unwrap();
        client.get(full_url)
    }

    fn requires_authentication(&self) -> bool {
        true
    }
}
