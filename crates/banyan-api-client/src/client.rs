use jsonwebtoken::EncodingKey;

use crate::requests::ApiRequest;

//pub type Request = Box<dyn ApiRequest>;
//
//pub type Response = Box<dyn ApiResponse>;

struct BearerToken;

pub struct Client {
    base_url: url::Url,
    client: reqwest::Client,

    ec_key: Option<EncodingKey>,
    bearer_token: Option<BearerToken>,
}

impl Client {
    fn bearer_token(&self) -> Option<String> {
        todo!()
    }

    pub fn new(base_url: reqwest::Url) -> Self {
        let mut default_headers = reqwest::header::HeaderMap::new();
        default_headers.insert("Content-Type", reqwest::header::HeaderValue::from_static("application/json"));

        let mut client = reqwest::Client::builder()
            .default_headers(default_headers)
            .user_agent("banyan-api-client/0.1.0")
            .build()
            .unwrap();

        Self {
            base_url,
            client,

            ec_key: None,
            bearer_token: None,
        }
    }

    pub async fn call<T: ApiRequest>(&self, request: T) -> Result<T::ResponseType, ClientError> {
        if request.requires_authentication() && !self.has_authentication() {
            return Err(ClientError::auth_unavailable());
        }

        let mut request_builder = request.build_request(&self.base_url, &self.client);

        if request.requires_authentication() {
            request_builder = request_builder.bearer_auth(self.bearer_token().unwrap());
        }

        let response = request_builder.send().await.map_err(ClientError::http_error)?;

        if response.status().is_success() {
            response.json::<T::ResponseType>().await.map_err(ClientError::bad_format)
        } else {
            let err = response.json::<T::ErrorType>().await.map_err(ClientError::bad_format)?;
            let err = Box::new(err) as Box<dyn std::error::Error + Send + Sync + 'static>;
            Err(ClientError::from(err))
        }
    }

    fn has_authentication(&self) -> bool {
        self.ec_key.is_some()
    }
}

pub struct ClientBuilder;

impl ClientBuilder {
    pub fn new() -> Self {
        Self
    }
}

#[non_exhaustive]
pub struct ClientError {
    kind: ClientErrorKind,
}

impl ClientError {
    fn auth_unavailable() -> Self {
        Self {
            kind: ClientErrorKind::AuthUnavailable,
        }
    }

    fn bad_format(err: reqwest::Error) -> Self {
        Self {
            kind: ClientErrorKind::ResponseFormatError(err),
        }
    }

    fn http_error(err: reqwest::Error) -> Self {
        Self {
            kind: ClientErrorKind::HttpClientError(err),
        }
    }
}

impl From<Box<dyn std::error::Error + Send + Sync + 'static>> for ClientError {
    fn from(err: Box<dyn std::error::Error + Send + Sync + 'static>) -> Self {
        Self {
            kind: ClientErrorKind::ApiResponseError(err),
        }
    }
}

#[non_exhaustive]
enum ClientErrorKind {
    ApiResponseError(Box<dyn std::error::Error + Send + Sync + 'static>),
    AuthUnavailable,
    HttpClientError(reqwest::Error),
    ResponseFormatError(reqwest::Error),
}
