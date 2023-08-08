use std::{error::Error, fmt::{self, Formatter, Display}};
use reqwest::{Url, RequestBuilder, Client};
use serde::{Serialize, Deserialize};
use crate::ApiRequest;

#[derive(Debug, Serialize)]
pub struct ListBuckets {}

impl ApiRequest for ListBuckets {
    type ResponseType = ListBucketsResponse;
    type ErrorType = ListBucketsError;

    fn build_request(self, base_url: &Url, client: &Client) -> RequestBuilder {
        let url = base_url.join("/api/v1/buckets").unwrap();
        client.get(url).json(&self)
    }

    fn requires_authentication(&self) -> bool { false }
}

#[derive(Debug, Deserialize)]
pub struct ListBucketsResponse {}

#[derive(Debug, Deserialize)]
#[non_exhaustive]
pub struct ListBucketsError {
    #[serde(rename = "error")]
    kind: ListBucketsErrorKind,
}

impl Display for ListBucketsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use ListBucketsErrorKind::*;

        let msg = match &self.kind {
            Unknown => "an unknown error occurred getting the bucket",
        };

        f.write_str(msg)
    }
}

impl Error for ListBucketsError {}

#[derive(Debug, Deserialize)]
#[non_exhaustive]
#[serde(tag = "type", rename_all = "snake_case")]
enum ListBucketsErrorKind {
    Unknown,
}
