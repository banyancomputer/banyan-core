use reqwest::{Client, RequestBuilder, Url};
use serde::Deserialize;
use uuid::Uuid;

use crate::requests::{ApiRequest, InfallibleError};

#[derive(Debug)]
pub struct WhoAmI;

impl ApiRequest for WhoAmI {
    type ResponseType = WhoAmIResponse;
    type ErrorType = InfallibleError;

    fn build_request(self, base_url: &Url, client: &Client) -> RequestBuilder {
        let full_url = base_url.join("/api/v1/auth/whoami").unwrap();
        client.get(full_url)
    }

    fn requires_authentication(&self) -> bool {
        true
    }
}

#[derive(Debug, Deserialize)]
pub struct WhoAmIResponse {
    pub account_id: Uuid,
}
