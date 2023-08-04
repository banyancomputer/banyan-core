use jsonwebtoken::{get_current_timestamp, EncodingKey};
use uuid::Uuid;

use crate::api_token::ApiToken;
use crate::requests::ApiRequest;

pub struct Credentials {
    account_id: Uuid,
    fingerprint: String,
    signing_key: EncodingKey,
}

pub struct Client {
    base_url: reqwest::Url,
    client: reqwest::Client,

    bearer_token: Option<(u64, String)>,
    credentials: Option<Credentials>,
}

impl Client {
    fn bearer_token(&mut self) -> Option<String> {
        match &self.bearer_token {
            // Good to go
            Some((exp, token)) if exp <= &(get_current_timestamp() - 15) => Some(token.clone()),
            // Either expired or not yet generated
            _ => {
                if let Some(credentials) = &self.credentials {
                    let api_token = ApiToken::new("banyan-platform", &credentials.account_id.to_string());
                    let expiration = api_token.expiration();
                    let signed_token =
                        api_token.sign(&credentials.fingerprint, &credentials.signing_key);

                    self.bearer_token = Some((expiration, signed_token.clone()));

                    return Some(signed_token);
                }

                None
            }
        }
    }

    pub fn set_credentials(
        &mut self,
        account_id: Uuid,
        fingerprint: String,
        signing_key: EncodingKey,
    ) {
        self.credentials = Some(Credentials {
            account_id,
            fingerprint,
            signing_key,
        });
    }

    pub fn new(base_url: reqwest::Url, credentials: Option<Credentials>) -> Self {
        let mut default_headers = reqwest::header::HeaderMap::new();
        default_headers.insert(
            "Content-Type",
            reqwest::header::HeaderValue::from_static("application/json"),
        );

        let client = reqwest::Client::builder()
            .default_headers(default_headers)
            .user_agent("banyan-api-client/0.1.0")
            .build()
            .unwrap();

        Self {
            base_url,
            client,

            bearer_token: None,
            credentials,
        }
    }

    pub async fn call<T: ApiRequest>(
        &mut self,
        request: T,
    ) -> Result<T::ResponseType, ClientError> {
        if request.requires_authentication() && !self.has_authentication() {
            return Err(ClientError::auth_unavailable());
        }

        let add_authentication = request.requires_authentication();
        let mut request_builder = request.build_request(&self.base_url, &self.client);

        if add_authentication {
            let bearer_token = self.bearer_token().unwrap();
            request_builder = request_builder.bearer_auth(bearer_token);
        }

        let response = request_builder
            .send()
            .await
            .map_err(ClientError::http_error)?;

        if response.status().is_success() {
            response
                .json::<T::ResponseType>()
                .await
                .map_err(ClientError::bad_format)
        } else {
            let err = response
                .json::<T::ErrorType>()
                .await
                .map_err(ClientError::bad_format)?;

            let err = Box::new(err) as Box<dyn std::error::Error + Send + Sync + 'static>;
            Err(ClientError::from(err))
        }
    }

    fn has_authentication(&self) -> bool {
        self.credentials.is_some()
    }
}

pub struct ClientBuilder {
    base_url: reqwest::Url,
    credentials: Option<Credentials>,
}

impl ClientBuilder {
    pub fn credentials(mut self, credentials: Credentials) -> Self {
        self.credentials = Some(credentials);
        self
    }

    pub fn build(self) -> Result<Client, &'static str> {
        Ok(Client::new(self.base_url, self.credentials))
    }

    pub fn base_url(mut self, url: reqwest::Url) -> Self {
        self.base_url = url;
        self
    }

    pub fn new() -> Self {
        Self {
            base_url: reqwest::Url::parse("http://127.0.0.1:3001").unwrap(),
            credentials: None,
        }
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
#[non_exhaustive]
pub struct ClientError {
    #[allow(dead_code)]
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

#[derive(Debug)]
#[non_exhaustive]
enum ClientErrorKind {
    ApiResponseError(Box<dyn std::error::Error + Send + Sync + 'static>),
    AuthUnavailable,
    HttpClientError(reqwest::Error),
    ResponseFormatError(reqwest::Error),
}
