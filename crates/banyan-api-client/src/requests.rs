use std::error::Error;
use std::fmt::{self, Display, Formatter};

use reqwest::{Client, RequestBuilder, Url};
use serde::de::DeserializeOwned;
use serde::Deserialize;

mod common;
mod create_bucket;
mod delete_bucket;
mod publish_bucket_metadata;
mod whoami;

pub use common::*;
pub use create_bucket::*;
pub use delete_bucket::*;
pub use publish_bucket_metadata::*;
pub use whoami::*;

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
