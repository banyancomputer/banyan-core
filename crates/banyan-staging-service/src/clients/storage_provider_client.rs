use http::{HeaderMap, HeaderValue};
use jwt_simple::prelude::*;
use reqwest::multipart::{Form, Part};
use reqwest::{Client, Response};
use serde::Serialize;
use url::Url;

pub struct StorageProviderClient {
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

impl StorageProviderClient {
    pub fn new(
        // service_signing_key: SigningKey,
        service_name: &str,
        platform_hostname: Url,
    ) -> Self {
        // let mut claims = Claims::create(Duration::from_secs(60))
        //     .with_audiences(HashSet::from_strings(&[platform_name]))
        //     .with_subject(service_name)
        //     .invalid_before(Clock::now_since_epoch() - Duration::from_secs(30));
        //
        // claims.create_nonce();
        // claims.issued_at = Some(Clock::now_since_epoch());
        // let bearer_token = service_signing_key.sign(claims).unwrap();
        let mut default_headers = HeaderMap::new();
        default_headers.insert("Content-Type", HeaderValue::from_static("application/json"));

        let client = Client::builder()
            .default_headers(default_headers)
            .build()
            .unwrap();

        Self {
            client,
            bearer_token: String::default(),
            platform_hostname,
        }
    }
    pub async fn upload_blocks(
        &self,
        content: Vec<u8>,
        host_url: String,
    ) -> Result<Response, reqwest::Error> {
        let path = "/api/v1/upload".to_string();
        let full_url = Url::parse(&host_url).unwrap().join(&path).unwrap();
        // Attach the CAR file to the request
        let content_len = content.len();
        let multipart_car = Part::stream(content)
            .mime_str("application/vnd.ipld.car; version=2")
            .unwrap();
        // Combine the two parts into a multipart form
        let multipart_form = Form::new().part("file", multipart_car);

        // post
        self.client
            .post(full_url)
            .multipart(multipart_form)
            .header(reqwest::header::CONTENT_LENGTH, content_len)
            .send()
            .await
    }
}
