use std::error::Error;
use std::fmt::{self, Display, Formatter};
use reqwest::{Client, RequestBuilder, Url};
use serde::de::DeserializeOwned;
use serde::Deserialize;

mod requests;
pub mod fake;
mod client;
mod api_token;
pub mod prelude;

// pub use common::*;
pub use requests::{
    common::*,
    publish_bucket_metadata::*,
    whoami::*
};

pub trait ApiRequest {
    type ResponseType: DeserializeOwned;
    type ErrorType: DeserializeOwned + Error + Send + Sync + 'static;

    fn build_request(self, base_url: &Url, client: &Client) -> RequestBuilder;
    fn requires_authentication(&self) -> bool;
}

/// Used for API responses that have no error state at the application level (client errors may
/// still occur).
#[derive(Debug, Deserialize)]
pub struct InfallibleError;

impl Display for InfallibleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("an infallible API query returned a failed response")
    }
}

impl std::error::Error for InfallibleError {}
