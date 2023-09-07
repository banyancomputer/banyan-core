use std::error::Error;
use std::fmt::{self, Display, Formatter};

use reqwest::{Client, RequestBuilder, Url};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::requests::{ApiRequest, InfallibleError};


#[derive(Debug)]
pub struct RegisterFakeAccount;

impl ApiRequest for RegisterFakeAccount {
    type ResponseType = RegisterFakeAccountResponse;
    type ErrorType = InfallibleError;

    fn build_request(self, base_url: &Url, client: &Client) -> RequestBuilder {
        let full_url = base_url.join("/api/v1/auth/create_fake_account").unwrap();
        client.get(full_url)
    }

    fn requires_authentication(&self) -> bool {
        false
    }
}

#[derive(Debug, Deserialize)]
pub struct RegisterFakeAccountResponse {
    pub id: Uuid,
    pub token: String,
}

#[derive(Debug)]
pub struct FakeRegisterDeviceKey {
    pub token: String,
    pub public_key: String,
}

impl ApiRequest for FakeRegisterDeviceKey {
    type ResponseType = FakeRegisterDeviceKeyResponse;
    type ErrorType = FakeRegisterDeviceKeyError;

    fn build_request(self, base_url: &Url, client: &Client) -> RequestBuilder {
        let inner_req = FakeRegisterDeviceKeyRequest {
            public_key: self.public_key,
        };

        let full_url = base_url
            .join("/api/v1/auth/fake_register_device_key")
            .unwrap();

        // We have to setup our own auth as we're not using the normal one for this request...
        client
            .post(full_url)
            .bearer_auth(&self.token)
            .json(&inner_req)
    }

    fn requires_authentication(&self) -> bool {
        false
    }
}

#[derive(Debug, Serialize)]
struct FakeRegisterDeviceKeyRequest {
    public_key: String,
}

#[derive(Debug, Deserialize)]
#[non_exhaustive]
pub struct FakeRegisterDeviceKeyError {
    #[serde(rename = "error")]
    kind: FakeRegisterDeviceKeyErrorKind,
}

impl Display for FakeRegisterDeviceKeyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use FakeRegisterDeviceKeyErrorKind::*;

        let msg = match &self.kind {
            InvalidPublicKey => "provided public key was invalid",
            KeyContextUnavailable => "key context was unavailable to process key",
            PersistenceFailed => "unable to persist changes on the server side",
        };

        f.write_str(msg)
    }
}

impl Error for FakeRegisterDeviceKeyError {}

#[derive(Debug, Deserialize)]
#[non_exhaustive]
#[serde(tag = "type", rename_all = "snake_case")]
enum FakeRegisterDeviceKeyErrorKind {
    InvalidPublicKey,
    KeyContextUnavailable,
    PersistenceFailed,
}

#[derive(Debug, Deserialize)]
pub struct FakeRegisterDeviceKeyResponse {
    pub id: Uuid,
    pub account_id: Uuid,
    pub fingerprint: String,
}
