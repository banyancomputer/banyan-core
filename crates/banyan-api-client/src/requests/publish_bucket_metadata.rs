use std::error::Error;
use std::path::PathBuf;
use std::fmt::{self, Display, Formatter};

use reqwest::{Client, RequestBuilder, Url};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::requests::{ApiRequest, MetadataState};

#[derive(Debug)]
pub struct PublishBucketMetadata {
    pub bucket_id: Uuid,
    pub metadata_path: PathBuf,
    pub expected_data_size: usize,
}

impl ApiRequest for PublishBucketMetadata {
    type ResponseType = PublishBucketMetadataResponse;
    type ErrorType = PublishBucketMetadataError;

    fn build_request(&self, base_url: &Url, client: &Client) -> RequestBuilder {
        let pbm_req = PublishBucketMetadataRequest {
            data_size: self.expected_data_size,
        };

        let full_url = base_url.join(format!("/api/v1/buckets/{}/publish", self.bucket_id).as_str()).unwrap();

        // todo: need to workaround reqwest's multipart limitations

        //let multipart_json = reqwest::multipart::Part::bytes(multipart_json_data.as_bytes().to_vec())
        //    .mime_str("application/json")
        //    .unwrap();

        //let multipart_car_data = "some random contents for the car file...";
        //let multipart_car = reqwest::multipart::Part::bytes(multipart_car_data.as_bytes().to_vec())
        //    .mime_str("application/vnd.ipld.car; version=2")
        //    .unwrap();

        client.post(full_url).json(&pbm_req)
    }

    fn requires_authentication(&self) -> bool {
        true
    }
}

#[derive(Debug, Serialize)]
struct PublishBucketMetadataRequest {
    data_size: usize,
}

#[derive(Debug, Deserialize)]
pub struct PublishBucketMetadataResponse {
    pub id: Uuid,
    pub state: MetadataState,

    pub storage_host: String,
    pub storage_authorization: String,
}

#[derive(Debug, Deserialize)]
#[non_exhaustive]
pub struct PublishBucketMetadataError {
    #[serde(rename = "error")]
    kind: PublishBucketMetadataErrorKind,
}

impl Display for PublishBucketMetadataError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use PublishBucketMetadataErrorKind::*;

        let msg = match &self.kind {
            Unknown => "an unknown error occurred publishing the metadata",
        };

        f.write_str(msg)
    }
}

impl Error for PublishBucketMetadataError {}

#[derive(Debug, Deserialize)]
#[non_exhaustive]
#[serde(tag = "type", rename_all = "snake_case")]
enum PublishBucketMetadataErrorKind {
    Unknown,
}
