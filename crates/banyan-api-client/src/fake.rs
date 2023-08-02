use std::error::Error;
use std::fmt::{self, Display, Formatter};

use openssl::ec::{EcGroup, EcKey};
use openssl::nid::Nid;
use openssl::pkey::{PKey, Private, Public};
use reqwest::{Client, RequestBuilder, Url};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::requests::{ApiRequest, InfallibleError};

pub fn create_private_ec_pem() -> String {
    let private_key: PKey<Private> = {
        let ec_group = EcGroup::from_curve_name(Nid::SECP384R1).unwrap();
        let ec_key = EcKey::generate(&ec_group).unwrap();
        ec_key.try_into().unwrap()
    };

    String::from_utf8(private_key.private_key_to_pem_pkcs8().unwrap()).unwrap()
}

pub fn fingerprint_public_pem(public_pem: &str) -> String {
    let public_key = PKey::public_key_from_pem(public_pem.as_bytes()).unwrap();

    let fingerprint_bytes = {
        use openssl::bn::BigNumContext;
        use openssl::ec::PointConversionForm;

        let ec_group = EcGroup::from_curve_name(Nid::SECP384R1).unwrap();
        let mut big_num_ctx = BigNumContext::new().unwrap();

        let ec_pub_key = public_key.ec_key().unwrap();
        let compressed_key = ec_pub_key
            .public_key()
            .to_bytes(&ec_group, PointConversionForm::COMPRESSED, &mut big_num_ctx)
            .unwrap();

        openssl::sha::sha1(&compressed_key)
    };

    fingerprint_bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<String>>()
        .join(":")
}

pub fn public_from_private(private_pem: &str) -> String {
    let private_key = PKey::private_key_from_pem(private_pem.as_bytes()).unwrap();

    let public_key: PKey<Public> = {
        let ec_group = EcGroup::from_curve_name(Nid::SECP384R1).unwrap();
        let priv_ec_key = private_key.ec_key().unwrap();
        let pub_ec_key: EcKey<Public> =
            EcKey::from_public_key(&ec_group, priv_ec_key.public_key()).unwrap();

        PKey::from_ec_key(pub_ec_key).unwrap()
    };

    String::from_utf8(public_key.public_key_to_pem().unwrap()).unwrap()
}

#[derive(Debug)]
pub struct RegisterFakeAccount;

impl ApiRequest for RegisterFakeAccount {
    type ResponseType = RegisterFakeAccountResponse;
    type ErrorType = InfallibleError;

    fn build_request(&self, base_url: &Url, client: &Client) -> RequestBuilder {
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

    fn build_request(&self, base_url: &Url, client: &Client) -> RequestBuilder {
        let inner_req = FakeRegisterDeviceKeyRequest {
            public_key: self.public_key.clone(),
        };

        let req_builder = inner_req.build_request(base_url, client);

        // We have to setup our own auth as we're not using the normal one for this request...
        req_builder.bearer_auth(&self.token)
    }

    fn requires_authentication(&self) -> bool {
        false
    }
}

#[derive(Debug, Serialize)]
struct FakeRegisterDeviceKeyRequest {
    public_key: String,
}

impl ApiRequest for FakeRegisterDeviceKeyRequest {
    type ResponseType = FakeRegisterDeviceKeyResponse;
    type ErrorType = FakeRegisterDeviceKeyError;

    fn build_request(&self, base_url: &Url, client: &Client) -> RequestBuilder {
        let full_url = base_url
            .join("/api/v1/auth/fake_register_device_key")
            .unwrap();
        client.post(full_url).json(self)
    }

    fn requires_authentication(&self) -> bool {
        false
    }
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
