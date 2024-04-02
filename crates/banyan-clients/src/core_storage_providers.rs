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
    ///ApiDeal
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "properties": {
    ///    "id": {
    ///      "type": "string"
    ///    },
    ///    "size": {
    ///      "type": "integer",
    ///      "format": "int64"
    ///    },
    ///    "state": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct ApiDeal {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub size: Option<i64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub state: Option<String>,
    }
    impl From<&ApiDeal> for ApiDeal {
        fn from(value: &ApiDeal) -> Self {
            value.clone()
        }
    }
    ///CompleteRedistributionError
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
    pub struct CompleteRedistributionError {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub msg: Option<String>,
    }
    impl From<&CompleteRedistributionError> for CompleteRedistributionError {
        fn from(value: &CompleteRedistributionError) -> Self {
            value.clone()
        }
    }
    ///CompleteRedistributionRequest
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "properties": {
    ///    "grant_id": {
    ///      "type": "string"
    ///    },
    ///    "normalized_cids": {
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct CompleteRedistributionRequest {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub grant_id: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub normalized_cids: Vec<String>,
    }
    impl From<&CompleteRedistributionRequest> for CompleteRedistributionRequest {
        fn from(value: &CompleteRedistributionRequest) -> Self {
            value.clone()
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
    ///MeterTrafficRequest
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "properties": {
    ///    "egress": {
    ///      "type": "integer",
    ///      "format": "int64"
    ///    },
    ///    "ingress": {
    ///      "type": "integer",
    ///      "format": "int64"
    ///    },
    ///    "slot": {
    ///      "type": "integer",
    ///      "format": "int64"
    ///    },
    ///    "user_id": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct MeterTrafficRequest {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub egress: Option<i64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub ingress: Option<i64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub slot: Option<i64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub user_id: Option<String>,
    }
    impl From<&MeterTrafficRequest> for MeterTrafficRequest {
        fn from(value: &MeterTrafficRequest) -> Self {
            value.clone()
        }
    }
    ///ProviderGrantResponse
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "properties": {
    ///    "token": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct ProviderGrantResponse {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub token: Option<String>,
    }
    impl From<&ProviderGrantResponse> for ProviderGrantResponse {
        fn from(value: &ProviderGrantResponse) -> Self {
            value.clone()
        }
    }
    ///PruneBlocksHookError
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
    pub struct PruneBlocksHookError {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub msg: Option<String>,
    }
    impl From<&PruneBlocksHookError> for PruneBlocksHookError {
        fn from(value: &PruneBlocksHookError) -> Self {
            value.clone()
        }
    }
    ///ReportHealth
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "properties": {
    ///    "build_profile": {
    ///      "type": "string"
    ///    },
    ///    "features": {
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "version": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct ReportHealth {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub build_profile: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub features: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub version: Option<String>,
    }
    impl From<&ReportHealth> for ReportHealth {
        fn from(value: &ReportHealth) -> Self {
            value.clone()
        }
    }
    ///ReportUploadError
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
    pub struct ReportUploadError {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub msg: Option<String>,
    }
    impl From<&ReportUploadError> for ReportUploadError {
        fn from(value: &ReportUploadError) -> Self {
            value.clone()
        }
    }
    ///ReportUploadRequest
    ///
    /// <details><summary>JSON schema</summary>
    ///
    /// ```json
    ///{
    ///  "type": "object",
    ///  "properties": {
    ///    "data_size": {
    ///      "type": "integer",
    ///      "format": "int64"
    ///    },
    ///    "normalized_cids": {
    ///      "type": "array",
    ///      "items": {
    ///        "type": "string"
    ///      }
    ///    },
    ///    "storage_authorization_id": {
    ///      "type": "string"
    ///    }
    ///  }
    ///}
    /// ```
    /// </details>
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct ReportUploadRequest {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub data_size: Option<i64>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        pub normalized_cids: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pub storage_authorization_id: Option<String>,
    }
    impl From<&ReportUploadRequest> for ReportUploadRequest {
        fn from(value: &ReportUploadRequest) -> Self {
            value.clone()
        }
    }
}
#[derive(Clone, Debug)]
/**Client for Banyan Core Service Storage Provider API

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
    /**Report the completion of an upload

Sends a `POST` request to `/hooks/report/{metadata_id}`

*/
    pub async fn report_upload<'a>(
        &'a self,
        metadata_id: &'a uuid::Uuid,
        body: &'a types::ReportUploadRequest,
    ) -> Result<ResponseValue<()>, Error<types::Error>> {
        let url = format!(
            "{}/hooks/report/{}", self.baseurl, encode_path(& metadata_id.to_string()),
        );
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
            204u16 => Ok(ResponseValue::empty(response)),
            400u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            500u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Report the health of a storage provider

Sends a `POST` request to `/hooks/report/health`

*/
    pub async fn report_health<'a>(
        &'a self,
        body: &'a types::ReportHealth,
    ) -> Result<ResponseValue<()>, Error<types::Error>> {
        let url = format!("{}/hooks/report/health", self.baseurl,);
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
            204u16 => Ok(ResponseValue::empty(response)),
            500u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Prune blocks that are no longer needed

Sends a `POST` request to `/hooks/prune`

*/
    pub async fn prune_blocks<'a>(
        &'a self,
        body: &'a Vec<String>,
    ) -> Result<ResponseValue<()>, Error<types::Error>> {
        let url = format!("{}/hooks/prune", self.baseurl,);
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
            204u16 => Ok(ResponseValue::empty(response)),
            400u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            500u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Complete the redistribution of blocks to a new storage host

Sends a `POST` request to `/hooks/redistribution/{metadata_id}`

*/
    pub async fn complete_redistribution<'a>(
        &'a self,
        metadata_id: &'a uuid::Uuid,
        body: &'a types::CompleteRedistributionRequest,
    ) -> Result<ResponseValue<()>, Error<types::Error>> {
        let url = format!(
            "{}/hooks/redistribution/{}", self.baseurl, encode_path(& metadata_id
            .to_string()),
        );
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
            204u16 => Ok(ResponseValue::empty(response)),
            400u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            500u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Grant a storage provider access to a storage host

Sends a `GET` request to `/api/v1/auth/provider_grant/{storage_host_id}`

Arguments:
- `storage_host_id`: The ID of the storage host to grant access to
*/
    pub async fn grant_provider_access<'a>(
        &'a self,
        storage_host_id: &'a str,
    ) -> Result<ResponseValue<types::ProviderGrantResponse>, Error<types::Error>> {
        let url = format!(
            "{}/api/v1/auth/provider_grant/{}", self.baseurl, encode_path(&
            storage_host_id.to_string()),
        );
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
            401u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            404u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            500u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Get all deals for the user

Sends a `GET` request to `/api/v1/deals`

*/
    pub async fn get_all_deals<'a>(
        &'a self,
    ) -> Result<ResponseValue<Vec<types::ApiDeal>>, Error<types::Error>> {
        let url = format!("{}/api/v1/deals", self.baseurl,);
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
    /**Get a single deal by ID

Sends a `GET` request to `/api/v1/deals/{deal_id}`

*/
    pub async fn get_single_deal<'a>(
        &'a self,
        deal_id: &'a uuid::Uuid,
    ) -> Result<ResponseValue<types::ApiDeal>, Error<types::Error>> {
        let url = format!(
            "{}/api/v1/deals/{}", self.baseurl, encode_path(& deal_id.to_string()),
        );
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
            404u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            500u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Accept a deal by ID

Sends a `POST` request to `/api/v1/deals/{deal_id}/accept`

*/
    pub async fn accept_deal<'a>(
        &'a self,
        deal_id: &'a uuid::Uuid,
    ) -> Result<ResponseValue<()>, Error<types::Error>> {
        let url = format!(
            "{}/api/v1/deals/{}/accept", self.baseurl, encode_path(& deal_id
            .to_string()),
        );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .post(url)
            .header(
                reqwest::header::ACCEPT,
                reqwest::header::HeaderValue::from_static("application/json"),
            )
            .build()?;
        let result = self.client.execute(request).await;
        let response = result?;
        match response.status().as_u16() {
            204u16 => Ok(ResponseValue::empty(response)),
            404u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            500u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Cancel a deal by ID

Sends a `POST` request to `/api/v1/deals/{deal_id}/cancel`

*/
    pub async fn cancel_deal<'a>(
        &'a self,
        deal_id: &'a uuid::Uuid,
    ) -> Result<ResponseValue<()>, Error<types::Error>> {
        let url = format!(
            "{}/api/v1/deals/{}/cancel", self.baseurl, encode_path(& deal_id
            .to_string()),
        );
        #[allow(unused_mut)]
        let mut request = self
            .client
            .post(url)
            .header(
                reqwest::header::ACCEPT,
                reqwest::header::HeaderValue::from_static("application/json"),
            )
            .build()?;
        let result = self.client.execute(request).await;
        let response = result?;
        match response.status().as_u16() {
            204u16 => Ok(ResponseValue::empty(response)),
            404u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            500u16 => {
                Err(Error::ErrorResponse(ResponseValue::from_response(response).await?))
            }
            _ => Err(Error::UnexpectedResponse(response)),
        }
    }
    /**Record the ingress and egress traffic for a storage provider

Sends a `POST` request to `/api/v1/metrics/traffic`

*/
    pub async fn meter_traffic<'a>(
        &'a self,
        body: &'a types::MeterTrafficRequest,
    ) -> Result<ResponseValue<()>, Error<types::Error>> {
        let url = format!("{}/api/v1/metrics/traffic", self.baseurl,);
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
            200u16 => Ok(ResponseValue::empty(response)),
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
