use crate::ApiRequest;
use reqwest::{Client, RequestBuilder, Url};
use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct GetBucket {
    pub bucket_id: Uuid,
}

impl ApiRequest for GetBucket {
    type ResponseType = GetBucketResponse;
    type ErrorType = GetBucketError;

    fn build_request(self, base_url: &Url, client: &Client) -> RequestBuilder {
        let path = format!("/api/v1/buckets/{}", self.bucket_id);
        let url = base_url.join(&path).unwrap();

        client.get(url).json(&self)
    }

    fn requires_authentication(&self) -> bool {
        true
    }
}

#[derive(Debug, Deserialize)]
pub struct GetBucketResponse {}

#[derive(Debug, Deserialize)]
#[non_exhaustive]
pub struct GetBucketError {
    #[serde(rename = "error")]
    kind: GetBucketErrorKind,
}

impl Display for GetBucketError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use GetBucketErrorKind::*;

        let msg = match &self.kind {
            Unknown => "an unknown error occurred getting the bucket",
        };

        f.write_str(msg)
    }
}

impl Error for GetBucketError {}

#[derive(Debug, Deserialize)]
#[non_exhaustive]
#[serde(tag = "type", rename_all = "snake_case")]
enum GetBucketErrorKind {
    Unknown,
}
