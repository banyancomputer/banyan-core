use std::error::Error;

use serde::de::DeserializeOwned;

mod create_bucket;

pub trait ApiRequest {
    type ResponseType: DeserializeOwned;
    type ErrorType: DeserializeOwned + Error + Send + Sync + 'static;

    fn build_request(&self, base_url: &url::Url, client: &reqwest::Client) -> reqwest::RequestBuilder;
    fn requires_authentication(&self) -> bool;
}
