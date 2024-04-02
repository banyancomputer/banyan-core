#[allow(unused_imports)]
pub use progenitor_client::{ByteStream, Error, ResponseValue};
#[allow(unused_imports)]
use progenitor_client::{encode_path, RequestBuilderExt};
#[allow(unused_imports)]
use reqwest::header::{HeaderMap, HeaderValue};
/// Types used as operation parameters and responses.
#[allow(clippy::all)]
pub mod types {
    use serde::{Deserialize, Serialize};
    #[allow(unused_imports)]
    use std::convert::TryFrom;
    /// Error types.
    pub mod error {
        /// Error from a TryFrom or FromStr implementation.
        pub struct ConversionError(std::borrow::Cow<'static, str>);
        impl std::error::Error for ConversionError {}
        impl std::fmt::Display for ConversionError {
            fn fmt(
                &self,
                f: &mut std::fmt::Formatter<'_>,
            ) -> Result<(), std::fmt::Error> {
                std::fmt::Display::fmt(&self.0, f)
            }
        }
        impl std::fmt::Debug for ConversionError {
            fn fmt(
                &self,
                f: &mut std::fmt::Formatter<'_>,
            ) -> Result<(), std::fmt::Error> {
                std::fmt::Debug::fmt(&self.0, f)
            }
        }
        impl From<&'static str> for ConversionError {
            fn from(value: &'static str) -> Self {
                Self(value.into())
            }
        }
        impl From<String> for ConversionError {
            fn from(value: String) -> Self {
                Self(value.into())
            }
        }
    }
    ///ApiDealsAdmin
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "properties": {
    ///    "accepted_at": {
    ///      "type": [
    ///        "string",
    ///        "null"
    ///      ],
    ///      "format": "date-time"
    ///    },
    ///    "accepted_by": {
    ///      "type": [
    ///        "string",
    ///        "null"
    ///      ]
    ///    },
    ///    "id": {
    ///      "type": "string"
    ///    },
    ///    "size": {
    ///      "type": "integer",
    ///      "format": "int64"
    ///    },
    ///    "state": {
    ///      "$ref": "#/components/schemas/DealState"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct ApiDealsAdmin {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub accepted_at: Option<chrono::DateTime<chrono::offset::Utc>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub accepted_by: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub size: Option<i64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub state: Option<DealState>,
    }
    impl From<&ApiDealsAdmin> for ApiDealsAdmin {
        fn from(value: &ApiDealsAdmin) -> Self {
            value.clone()
        }
    }
    ///ApiSelectedStorageHostAdmin
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "properties": {
    ///    "available_storage": {
    ///      "type": "integer",
    ///      "format": "int64"
    ///    },
    ///    "fingerprint": {
    ///      "type": "string"
    ///    },
    ///    "id": {
    ///      "type": "string"
    ///    },
    ///    "name": {
    ///      "type": "string"
    ///    },
    ///    "pem": {
    ///      "type": "string",
    ///      "format": "pem"
    ///    },
    ///    "url": {
    ///      "type": "string"
    ///    },
    ///    "used_storage": {
    ///      "type": "integer",
    ///      "format": "int64"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct ApiSelectedStorageHostAdmin {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub available_storage: Option<i64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub fingerprint: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub pem: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub url: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub used_storage: Option<i64>,
    }
    impl From<&ApiSelectedStorageHostAdmin> for ApiSelectedStorageHostAdmin {
        fn from(value: &ApiSelectedStorageHostAdmin) -> Self {
            value.clone()
        }
    }
    ///DealState
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "string",
    ///  "enum": [
    ///    "Pending",
    ///    "Accepted",
    ///    "Rejected"
    ///  ]
    ///}
    /// ```
    /// </details>
    #[derive(
        Clone,
        Copy,
        Debug,
        Deserialize,
        Eq,
        Hash,
        Ord,
        PartialEq,
        PartialOrd,
        Serialize
    )]
    pub enum DealState {
        Pending,
        Accepted,
        Rejected,
    }
    impl From<&DealState> for DealState {
        fn from(value: &DealState) -> Self {
            value.clone()
        }
    }
    impl ToString for DealState {
        fn to_string(&self) -> String {
            match *self {
                Self::Pending => "Pending".to_string(),
                Self::Accepted => "Accepted".to_string(),
                Self::Rejected => "Rejected".to_string(),
            }
        }
    }
    impl std::str::FromStr for DealState {
        type Err = self::error::ConversionError;
        fn from_str(value: &str) -> Result<Self, self::error::ConversionError> {
            match value {
                "Pending" => Ok(Self::Pending),
                "Accepted" => Ok(Self::Accepted),
                "Rejected" => Ok(Self::Rejected),
                _ => Err("invalid value".into()),
            }
        }
    }
    impl std::convert::TryFrom<&str> for DealState {
        type Error = self::error::ConversionError;
        fn try_from(value: &str) -> Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl std::convert::TryFrom<&String> for DealState {
        type Error = self::error::ConversionError;
        fn try_from(value: &String) -> Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    impl std::convert::TryFrom<String> for DealState {
        type Error = self::error::ConversionError;
        fn try_from(value: String) -> Result<Self, self::error::ConversionError> {
            value.parse()
        }
    }
    ///Error
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "properties": {
    ///    "msg": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Error {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub msg: Option<String>,
    }
    impl From<&Error> for Error {
        fn from(value: &Error) -> Self {
            value.clone()
        }
    }
    ///SelectedStorageHostRequest
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "properties": {
    ///    "available_storage": {
    ///      "type": "integer",
    ///      "format": "int64"
    ///    },
    ///    "name": {
    ///      "type": "string"
    ///    },
    ///    "region": {
    ///      "type": "string"
    ///    },
    ///    "url": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct SelectedStorageHostRequest {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub available_storage: Option<i64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub name: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub region: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub url: Option<String>,
    }
    impl From<&SelectedStorageHostRequest> for SelectedStorageHostRequest {
        fn from(value: &SelectedStorageHostRequest) -> Self {
            value.clone()
        }
    }
}
#[derive(Clone, Debug)]
/**Client for Banyan Core Service Admin API

Version: 1.0.0*/
pub struct Client {
    pub(crate) baseurl: String,
    pub(crate) client: reqwest::Client,
}
impl Client {
    /// Create a new client.
    ///
    /// `baseurl` is the base URL provided to the internal
    /// `reqwest::Client`, and should include a scheme and hostname,
    /// as well as port and a path stem if applicable.
    pub fn new(baseurl: &str) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let client = {
            let dur = std::time::Duration::from_secs(15);
            reqwest::ClientBuilder::new().connect_timeout(dur).timeout(dur)
        };
        #[cfg(target_arch = "wasm32")]
        let client = reqwest::ClientBuilder::new();
        Self::new_with_client(baseurl, client.build().unwrap())
    }
    /// Construct a new client with an existing `reqwest::Client`,
    /// allowing more control over its configuration.
    ///
    /// `baseurl` is the base URL provided to the internal
    /// `reqwest::Client`, and should include a scheme and hostname,
    /// as well as port and a path stem if applicable.
    pub fn new_with_client(baseurl: &str, client: reqwest::Client) -> Self {
        Self {
            baseurl: baseurl.to_string(),
            client,
        }
    }
    /// Get the base URL to which requests are made.
    pub fn baseurl(&self) -> &String {
        &self.baseurl
    }
    /// Get the internal `reqwest::Client` used to make requests.
    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }
    /// Get the version of this API.
    ///
    /// This string is pulled directly from the source OpenAPI
    /// document and may be in any format the API selects.
    pub fn api_version(&self) -> &'static str {
        "1.0.0"
    }
}
#[allow(clippy::all)]
impl Client {
    /**Get all deals

Sends a `GET` request to `/admin/deals`

*/
    pub async fn get_all_deals<'a>(
        &'a self,
    ) -> Result<ResponseValue<Vec<types::ApiDealsAdmin>>, Error<types::Error>> {
        let url = format!("{}/admin/deals", self.baseurl,);
        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                reqwest::header::ACCEPT,
                reqwest::header::HeaderValue::from_static("application/json"),
            )
            .build()?;
        let result = self.client.execute(request).await;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            500u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Get all storage providers

Sends a `GET` request to `/admin/providers`

*/
    pub async fn get_all_storage_hosts<'a>(
        &'a self,
    ) -> Result<
        ResponseValue<Vec<types::ApiSelectedStorageHostAdmin>>,
        Error<types::Error>,
    > {
        let url = format!("{}/admin/providers", self.baseurl,);
        #[allow(unused_mut)]
        let mut request = self
            .client
            .get(url)
            .header(
                reqwest::header::ACCEPT,
                reqwest::header::HeaderValue::from_static("application/json"),
            )
            .build()?;
        let result = self.client.execute(request).await;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            500u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Create a new storage provider

Sends a `POST` request to `/admin/providers`

*/
    pub async fn create_storage_host<'a>(
        &'a self,
        body: &'a types::SelectedStorageHostRequest,
    ) -> Result<ResponseValue<types::ApiSelectedStorageHostAdmin>, Error<types::Error>> {
        let url = format!("{}/admin/providers", self.baseurl,);
        #[allow(unused_mut)]
        let mut request = self
            .client
            .post(url)
            .header(
                reqwest::header::ACCEPT,
                reqwest::header::HeaderValue::from_static("application/json"),
            )
            .json(&body)
            .build()?;
        let result = self.client.execute(request).await;
        let response = result?;
        match response.status().as_u16() {
            200u16 => ResponseValue::from_response(response).await,
            400u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            500u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
}
/// Items consumers will typically use such as the Client.
pub mod prelude {
    #[allow(unused_imports)]
    pub use super::Client;
}
