use std::fmt::{self, Display, Formatter};

use reqwest::{Client, RequestBuilder, Url};
use serde::Deserialize;
use uuid::Uuid;

use crate::requests::ApiRequest;

pub struct WhoAmI;

#[derive(Debug, Deserialize)]
pub struct WhoAmIError {
    #[serde(flatten)]
    kind: WhoAmIErrorKind,
}

impl ApiRequest for WhoAmI {
    type ResponseType = WhoAmIResponse;
    type ErrorType = WhoAmIError;

    fn build_request(&self, base_url: &Url, client: &Client) -> RequestBuilder {
        let full_url = base_url.join("/api/v1/auth/whoami").unwrap();
        client.get(full_url)
    }

    fn requires_authentication(&self) -> bool {
        true
    }
}

#[derive(Deserialize)]
pub struct WhoAmIResponse {
    pub account_id: Uuid,
}

impl Display for WhoAmIError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use WhoAmIErrorKind::*;

        let msg = match &self.kind {
            Unknown(_) => "an unknown error occurred",
        };

        f.write_str(msg)
    }
}

impl std::error::Error for WhoAmIError {
    //fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    //    use WhoAmIErrorKind::*;

    //    match &self.kind {
    //        Unknown(msg) => Some(msg),
    //    }
    //}
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "error")]
enum WhoAmIErrorKind {
    Unknown(String),
}
