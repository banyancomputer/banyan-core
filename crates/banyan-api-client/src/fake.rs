use reqwest::{Client, RequestBuilder, Url};
use serde::Deserialize;
use uuid::Uuid;

use crate::requests::{ApiRequest, InfallibleError};

#[derive(Debug)]
pub struct RegisterFakeAccount;

impl ApiRequest for RegisterFakeAccount {
    type ResponseType = RegisterFakeAccountResponse;
    type ErrorType = InfallibleError;

    fn build_request(&self, base_url: &Url, client: &Client) -> RequestBuilder {
        let full_url = base_url.join("/api/v1/auth/create_fake_account").unwrap();
        client.get(full_url)
    }

    fn requires_authentication(&self) -> bool {
        false
    }
}

#[derive(Debug, Deserialize)]
pub struct RegisterFakeAccountResponse {
    pub id: Uuid,
}
