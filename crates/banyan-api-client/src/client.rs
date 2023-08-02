use jsonwebtoken::EncodingKey;

use crate::requests::ApiRequest;

//pub type Request = Box<dyn ApiRequest>;
//
//pub type Response = Box<dyn ApiResponse>;

struct BearerToken;

pub struct Client {
    base_url: url::Url,

    ec_key: Option<EncodingKey>,
    bearer_token: Option<BearerToken>,
}

impl Client {
    pub async fn call<T: ApiRequest>(&self, request: T) -> Result<T::ResponseType, ClientError> {
        if request.requires_authentication() && !self.has_authentication() {
            return Err(ClientError::auth_required());
        }

        let request_builder = request.build_request(&self.base_url);
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
    fn auth_required() -> Self {
        Self {
            kind: ClientErrorKind::AuthRequired,
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
    AuthRequired,
    HttpClientError(reqwest::Error),
    ResponseFormatError(reqwest::Error),
}
